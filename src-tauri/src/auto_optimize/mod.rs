use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::benchmark::run_short_benchmark;
use crate::error::{AppError, AppResult};
use crate::hardware::SystemReport;
use crate::tweaks::{
    all_tweaks, apply_tweak_by_id, detect_tweak, hardware_matches, revert_single_tweak,
    DetectedState, ImpactLevel, TweakDefinition,
};

static CANCEL: AtomicBool = AtomicBool::new(false);
static RUNNING: AtomicBool = AtomicBool::new(false);
static SESSION: Mutex<Option<AutoOptimizeSession>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchStats {
    pub avg_fps: f32,
    pub p1_fps: f32,
    pub jitter_ms: f64,
    pub variance_fps: f64,
    pub source: String,
    pub runs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakTrialResult {
    pub tweak_id: String,
    pub tweak_name: String,
    pub verdict: String,
    pub delta_avg_fps: f32,
    pub delta_p1_fps: f32,
    pub delta_jitter_ms: f64,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoOptimizeReport {
    pub id: String,
    pub started_at: String,
    pub finished_at: String,
    pub cancelled: bool,
    pub baseline: BenchStats,
    pub final_stats: BenchStats,
    pub kept_tweak_ids: Vec<String>,
    pub reverted_tweak_ids: Vec<String>,
    pub skipped_tweak_ids: Vec<String>,
    pub trials: Vec<TweakTrialResult>,
    pub summary: String,
}

#[derive(Debug, Clone)]
struct AutoOptimizeSession {
    pub report_id: String,
    pub applied_during_run: Vec<String>,
    pub pre_run_enabled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoOptimizeProgress {
    pub phase: String,
    pub message: String,
    pub current: u32,
    pub total: u32,
    pub eta_secs: u32,
    pub live_fps: f32,
    pub live_jitter_ms: f64,
}

pub fn cancel_auto_optimize() -> AppResult<()> {
    CANCEL.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn load_last_report() -> AppResult<Option<AutoOptimizeReport>> {
    let p = report_path()?;
    if !p.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&fs::read_to_string(p)?)?))
}

fn report_path() -> AppResult<PathBuf> {
    Ok(crate::backup::data_dir()?.join("auto_optimize_last.json"))
}

fn save_report(report: &AutoOptimizeReport) -> AppResult<()> {
    fs::write(report_path()?, serde_json::to_string_pretty(report)?)?;
    Ok(())
}

fn emit_progress(app: &AppHandle, progress: AutoOptimizeProgress) {
    let _ = app.emit("auto-optimize-progress", progress);
}

fn impact_rank(level: ImpactLevel) -> u8 {
    match level {
        ImpactLevel::High => 3,
        ImpactLevel::Medium => 2,
        ImpactLevel::Low => 1,
    }
}

pub fn candidate_tweaks(report: &SystemReport) -> Vec<TweakDefinition> {
    let mut list: Vec<TweakDefinition> = all_tweaks()
        .into_iter()
        .filter(|d| hardware_matches(d, report))
        .filter(|d| !d.guidance_only)
        .filter(|d| d.profiles.contains(&crate::tweaks::OptimizationProfile::Competitive))
        .filter(|d| detect_tweak(d) != DetectedState::On)
        .filter(|d| detect_tweak(d) != DetectedState::NotApplicable)
        .collect();
    list.sort_by(|a, b| impact_rank(b.impact).cmp(&impact_rank(a.impact)));
    list
}

pub fn run_multi_bench(app: Option<&AppHandle>, runs: u32, seconds_per_run: u32) -> AppResult<BenchStats> {
    if mock_bench_enabled() {
        let mut avgs = Vec::new();
        let mut p1s = Vec::new();
        let mut jitters = Vec::new();
        for r in 0..runs {
            let sample = mock_bench_stats(1);
            avgs.push(sample.avg_fps);
            p1s.push(sample.p1_fps);
            jitters.push(sample.jitter_ms);
            if let Some(handle) = app {
                emit_progress(
                    handle,
                    AutoOptimizeProgress {
                        phase: "benchmark".into(),
                        message: format!("Mock benchmark run {}/{}", r + 1, runs),
                        current: r + 1,
                        total: runs,
                        eta_secs: 0,
                        live_fps: sample.avg_fps,
                        live_jitter_ms: sample.jitter_ms,
                    },
                );
            }
        }
        return Ok(aggregate_samples(&avgs, &p1s, &jitters, "mock".into(), runs));
    }
    let mut avgs = Vec::new();
    let mut p1s = Vec::new();
    let mut jitters = Vec::new();
    let mut source = String::new();
    for r in 0..runs {
        if CANCEL.load(Ordering::SeqCst) {
            break;
        }
        let sample = run_short_benchmark(app.cloned(), seconds_per_run)?;
        if source.is_empty() {
            source = sample.source.clone();
        }
        avgs.push(sample.avg_fps);
        p1s.push(sample.p1_fps);
        jitters.push(sample.jitter_ms);
        if let Some(handle) = app {
            emit_progress(
                handle,
                AutoOptimizeProgress {
                    phase: "benchmark".into(),
                    message: format!("Benchmark run {}/{}", r + 1, runs),
                    current: r + 1,
                    total: runs,
                    eta_secs: (runs - r - 1) * seconds_per_run,
                    live_fps: sample.avg_fps,
                    live_jitter_ms: sample.jitter_ms,
                },
            );
        }
    }
    if avgs.is_empty() {
        return Err(AppError::msg("No benchmark samples collected"));
    }
    Ok(aggregate_samples(&avgs, &p1s, &jitters, source, runs))
}

fn aggregate_samples(
    avgs: &[f32],
    p1s: &[f32],
    jitters: &[f64],
    source: String,
    runs: u32,
) -> BenchStats {
    let avg_fps = mean_f32(avgs);
    let p1_fps = mean_f32(p1s);
    let jitter_ms = jitters.iter().sum::<f64>() / jitters.len() as f64;
    let variance_fps = variance_f32(avgs);
    BenchStats {
        avg_fps,
        p1_fps,
        jitter_ms,
        variance_fps,
        source,
        runs,
    }
}

fn mean_f32(v: &[f32]) -> f32 {
    v.iter().sum::<f32>() / v.len() as f32
}

fn variance_f32(v: &[f32]) -> f64 {
    let m = mean_f32(v) as f64;
    v.iter().map(|x| {
        let d = *x as f64 - m;
        d * d
    })
    .sum::<f64>()
        / v.len() as f64
}

pub fn compare_verdict(before: &BenchStats, after: &BenchStats) -> (String, String) {
    let d_avg = after.avg_fps - before.avg_fps;
    let d_p1 = after.p1_fps - before.p1_fps;
    let d_jit = after.jitter_ms - before.jitter_ms;
    let noise = (before.variance_fps.sqrt().max(1.0) as f32) * 0.5;

    if d_avg < -noise && d_p1 < -noise {
        return (
            "revert".into(),
            format!("FPS regression avg {d_avg:+.1} p1 {d_p1:+.1}"),
        );
    }
    if d_avg > noise || d_p1 > noise {
        return (
            "keep".into(),
            format!("Measured gain avg {d_avg:+.1} p1 {d_p1:+.1}"),
        );
    }
    if d_jit < -0.05 {
        return (
            "keep".into(),
            format!("Neutral FPS, lower jitter {d_jit:+.3} ms"),
        );
    }
    if d_jit > 0.15 {
        return (
            "revert".into(),
            format!("Higher frametime variance {d_jit:+.3} ms"),
        );
    }
    (
        "keep".into(),
        format!("Within noise — keeping (avg {d_avg:+.1}, p1 {d_p1:+.1})"),
    )
}

pub fn run_auto_optimize(app: AppHandle, report: SystemReport) -> AppResult<AutoOptimizeReport> {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return Err(AppError::msg("Auto Optimize is already running"));
    }
    CANCEL.store(false, Ordering::SeqCst);

    let report_id = uuid::Uuid::new_v4().to_string();
    let started_at = Utc::now().to_rfc3339();
    let pre_run_enabled: Vec<String> = all_tweaks()
        .into_iter()
        .filter(|d| hardware_matches(d, &report))
        .filter(|d| detect_tweak(d) == DetectedState::On)
        .map(|d| d.id)
        .collect();

    {
        let mut guard = SESSION.lock().map_err(|_| AppError::msg("session lock"))?;
        *guard = Some(AutoOptimizeSession {
            report_id: report_id.clone(),
            applied_during_run: vec![],
            pre_run_enabled,
        });
    }

    let runs = if mock_bench_enabled() { 2 } else { 3 };
    let seconds = if mock_bench_enabled() { 2 } else { 8 };

    emit_progress(
        &app,
        AutoOptimizeProgress {
            phase: "baseline".into(),
            message: "Running baseline benchmarks…".into(),
            current: 0,
            total: 0,
            eta_secs: runs * seconds,
            live_fps: 0.0,
            live_jitter_ms: 0.0,
        },
    );

    let baseline = run_multi_bench(Some(&app), runs, seconds)?;
    let candidates = candidate_tweaks(&report);
    let total = candidates.len() as u32;

    let mut trials = Vec::new();
    let mut kept = Vec::new();
    let mut reverted = Vec::new();
    let mut skipped = Vec::new();
    let mut reference = baseline.clone();

    for (idx, tweak) in candidates.iter().enumerate() {
        if CANCEL.load(Ordering::SeqCst) {
            break;
        }
        emit_progress(
            &app,
            AutoOptimizeProgress {
                phase: "tweak".into(),
                message: format!("Testing: {}", tweak.name),
                current: idx as u32 + 1,
                total,
                eta_secs: (total - idx as u32) * runs * seconds,
                live_fps: baseline.avg_fps,
                live_jitter_ms: baseline.jitter_ms,
            },
        );

        if let Err(e) = apply_tweak_by_id(&tweak.id, &report) {
            skipped.push(tweak.id.clone());
            trials.push(TweakTrialResult {
                tweak_id: tweak.id.clone(),
                tweak_name: tweak.name.clone(),
                verdict: "skip".into(),
                delta_avg_fps: 0.0,
                delta_p1_fps: 0.0,
                delta_jitter_ms: 0.0,
                detail: format!("Apply failed: {e}"),
            });
            continue;
        }

        if let Ok(mut guard) = SESSION.lock() {
            if let Some(s) = guard.as_mut() {
                s.applied_during_run.push(tweak.id.clone());
            }
        }

        let after = run_multi_bench(Some(&app), runs, seconds)?;
        let (verdict, detail) = compare_verdict(&reference, &after);
        let trial = TweakTrialResult {
            tweak_id: tweak.id.clone(),
            tweak_name: tweak.name.clone(),
            verdict: verdict.clone(),
            delta_avg_fps: after.avg_fps - reference.avg_fps,
            delta_p1_fps: after.p1_fps - reference.p1_fps,
            delta_jitter_ms: after.jitter_ms - reference.jitter_ms,
            detail,
        };
        trials.push(trial);

        if verdict == "revert" {
            let _ = revert_single_tweak(&tweak.id);
            reverted.push(tweak.id.clone());
        } else {
            kept.push(tweak.id.clone());
            reference = after;
        }
    }

    let cancelled = CANCEL.load(Ordering::SeqCst);
    if cancelled {
        rollback_session(&report)?;
    }

    let final_stats = if cancelled {
        baseline.clone()
    } else {
        emit_progress(
            &app,
            AutoOptimizeProgress {
                phase: "final".into(),
                message: "Final verification benchmark…".into(),
                current: total,
                total,
                eta_secs: runs * seconds,
                live_fps: baseline.avg_fps,
                live_jitter_ms: baseline.jitter_ms,
            },
        );
        run_multi_bench(Some(&app), runs, seconds)?
    };

    let finished_at = Utc::now().to_rfc3339();
    let summary = format!(
        "Baseline {:.1} avg / {:.1} 1% → Final {:.1} avg / {:.1} 1% · kept {} · reverted {} · skipped {}",
        baseline.avg_fps,
        baseline.p1_fps,
        final_stats.avg_fps,
        final_stats.p1_fps,
        kept.len(),
        reverted.len(),
        skipped.len()
    );

    let out = AutoOptimizeReport {
        id: report_id,
        started_at,
        finished_at,
        cancelled,
        baseline,
        final_stats,
        kept_tweak_ids: kept,
        reverted_tweak_ids: reverted,
        skipped_tweak_ids: skipped,
        trials,
        summary,
    };
    save_report(&out)?;

    RUNNING.store(false, Ordering::SeqCst);
    SESSION.lock().map_err(|_| AppError::msg("session lock"))?.take();
    let _ = app.emit("auto-optimize-complete", out.clone());
    Ok(out)
}

fn rollback_session(_report: &SystemReport) -> AppResult<()> {
    let applied = SESSION
        .lock()
        .map_err(|_| AppError::msg("session lock"))?
        .as_ref()
        .map(|s| s.applied_during_run.clone())
        .unwrap_or_default();
    for id in applied {
        let _ = revert_single_tweak(&id);
    }
    Ok(())
}

pub fn mock_bench_enabled() -> bool {
    std::env::var("BOOSTX_MOCK_BENCH")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

static MOCK_BENCH_SEQ: AtomicU32 = AtomicU32::new(0);

fn mock_bench_stats(runs: u32) -> BenchStats {
    let seq = MOCK_BENCH_SEQ.fetch_add(1, Ordering::SeqCst);
    let bump = (seq % 5) as f32 * 2.5;
    BenchStats {
        avg_fps: 280.0 + bump + runs as f32 * 0.1,
        p1_fps: 238.0 + bump * 0.8,
        jitter_ms: 0.92 - (seq % 3) as f64 * 0.02,
        variance_fps: 4.0,
        source: "mock".into(),
        runs,
    }
}

/// Multi-run aggregation + mock bench path (used when `BOOSTX_MOCK_BENCH=1` in CI).
pub fn mock_benchmark_selftest() -> AppResult<()> {
    if !mock_bench_enabled() {
        return Ok(());
    }
    MOCK_BENCH_SEQ.store(0, Ordering::SeqCst);
    let a = run_multi_bench(None, 3, 1)?;
    let b = run_multi_bench(None, 3, 1)?;
    if a.runs != 3 || b.runs != 3 {
        return Err(AppError::msg("mock_benchmark_selftest: expected 3 runs"));
    }
    if a.source != "mock" {
        return Err(AppError::msg("mock_benchmark_selftest: expected mock source"));
    }
    if a.variance_fps <= 0.0 {
        return Err(AppError::msg("mock_benchmark_selftest: variance should be positive"));
    }
    Ok(())
}

/// Deterministic unit test for verdict logic (no Windows required).
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdict_logic() {
        logic_selftest().expect("verdict logic");
    }
}

pub fn logic_selftest() -> AppResult<()> {
    let base = BenchStats {
        avg_fps: 300.0,
        p1_fps: 250.0,
        jitter_ms: 1.0,
        variance_fps: 9.0,
        source: "mock".into(),
        runs: 2,
    };
    let better = BenchStats {
        avg_fps: 310.0,
        p1_fps: 255.0,
        jitter_ms: 0.95,
        variance_fps: 9.0,
        source: "mock".into(),
        runs: 2,
    };
    let worse = BenchStats {
        avg_fps: 280.0,
        p1_fps: 220.0,
        jitter_ms: 1.2,
        variance_fps: 9.0,
        source: "mock".into(),
        runs: 2,
    };
    let (v, _) = compare_verdict(&base, &better);
    if v != "keep" {
        return Err(AppError::msg("logic_selftest: expected keep on better bench"));
    }
    let (v2, _) = compare_verdict(&base, &worse);
    if v2 != "revert" {
        return Err(AppError::msg("logic_selftest: expected revert on worse bench"));
    }
    Ok(())
}

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::history::{
    append, BenchmarkHistory, BenchmarkHistoryEntry, BenchmarkPhase, SecondaryBenchmarkDetails,
};
use super::pipeline::{save_last, BenchmarkPipelineLog};
use super::suite::{bench_cpu_multi, bench_cpu_single, bench_memory, bench_storage, bench_timer_jitter};
use crate::error::{AppError, AppResult};
use crate::benchmark::live::{emit_tick, BenchmarkLiveTick};
use crate::benchmark::telemetry::sample_host_telemetry;
use crate::games::{find_cs2_install, install_boostx_bench_cfg};
use tauri::AppHandle;

const BENCH_CAPTURE_SECONDS: u32 = 45;
const MIN_LIVE_SAMPLES: usize = 120;
const CS2_PROCESS_WAIT_SECS: u64 = 120;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BenchmarkPhaseInput {
    Baseline,
    AfterTweaks,
    AfterBios,
    Custom,
}

impl From<BenchmarkPhaseInput> for BenchmarkPhase {
    fn from(value: BenchmarkPhaseInput) -> Self {
        match value {
            BenchmarkPhaseInput::Baseline => BenchmarkPhase::Baseline,
            BenchmarkPhaseInput::AfterTweaks => BenchmarkPhase::AfterTweaks,
            BenchmarkPhaseInput::AfterBios => BenchmarkPhase::AfterBios,
            BenchmarkPhaseInput::Custom => BenchmarkPhase::Custom,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cs2CompetitiveResult {
    pub entry: BenchmarkHistoryEntry,
    pub history: BenchmarkHistory,
    pub delta_avg_fps_vs_previous: Option<f32>,
    pub delta_p1_fps_vs_previous: Option<f32>,
    pub delta_avg_fps_vs_starting: Option<f32>,
    pub delta_p1_fps_vs_starting: Option<f32>,
    pub summary: String,
    pub pipeline_log: BenchmarkPipelineLog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FrameMetrics {
    avg_fps: f32,
    p1_fps: f32,
    p01_fps: f32,
    jitter_ms: f64,
}

struct CaptureOutcome {
    source: String,
    metrics: FrameMetrics,
    sample_count: usize,
    duration_secs: f64,
    note: String,
}

pub fn run_competitive_benchmark(
    app: Option<AppHandle>,
    phase: BenchmarkPhaseInput,
    label: String,
    active_tweak_ids: Vec<String>,
    bios_summary: String,
) -> AppResult<Cs2CompetitiveResult> {
    let run_id = uuid::Uuid::new_v4().to_string();
    let mut log = BenchmarkPipelineLog::new(run_id.clone());
    log.step("Benchmark run started — fresh capture (no cached FPS reuse)");

    let timer_us = bench_timer_jitter();
    let (mem_bw, mem_lat) = bench_memory();
    let cpu_single = bench_cpu_single();
    let cpu_multi = bench_cpu_multi();
    let (read_mbps, write_mbps) = bench_storage();
    log.step(format!(
        "System micro-suite: cpu1t={cpu_single} cpuMt={cpu_multi} memLat={mem_lat:.1}us"
    ));

    let capture = match try_live_cs2_presentmon_capture(app.clone(), &run_id, &mut log) {
        Ok(out) => {
            log.step(format!(
                "cs2_live OK — {} samples over {:.1}s",
                out.sample_count,
                out.duration_secs
            ));
            out
        }
        Err(e) => {
            log.step(format!("cs2_live failed: {e}"));
            log.step("Falling back to timed micro-benchmark (not in-game FPS)");
            let out = measured_microbenchmark_capture(app, &mut log)?;
            CaptureOutcome {
                source: "cs2_microbench".into(),
                note: format!(
                    "CS2 PresentMon capture failed ({e}). Timed on-CPU micro-benchmark only — not comparable to in-game FPS."
                ),
                ..out
            }
        }
    };

    log.finish(&capture.source);
    save_last(&log)?;

    let input_score = estimate_input_path_score(timer_us);
    let system_latency = estimate_system_latency_score(timer_us, mem_lat);

    let entry_id = uuid::Uuid::new_v4().to_string();
    let entry = BenchmarkHistoryEntry {
        id: entry_id,
        captured_at: chrono::Utc::now().to_rfc3339(),
        phase: phase.into(),
        label,
        source: capture.source.clone(),
        cs2_avg_fps: capture.metrics.avg_fps,
        cs2_fps_p1: capture.metrics.p1_fps,
        cs2_fps_p01: capture.metrics.p01_fps,
        frametime_jitter_ms: capture.metrics.jitter_ms,
        system_latency_score: system_latency,
        timer_latency_us: timer_us,
        input_path_score: input_score,
        cpu_single_thread_score: cpu_single,
        memory_latency_us: mem_lat,
        active_tweak_ids,
        bios_summary,
        frame_sample_count: capture.sample_count as u32,
        capture_duration_secs: capture.duration_secs as f32,
        run_id: run_id.clone(),
        details: SecondaryBenchmarkDetails {
            cpu_single,
            cpu_multi,
            memory_bandwidth: mem_bw,
            storage_read_mbps: read_mbps,
            storage_write_mbps: write_mbps,
        },
    };

    let prev = super::history::load()?;
    let previous = prev.entries.last();
    let delta_avg = previous.map(|e| entry.cs2_avg_fps - e.cs2_avg_fps);
    let delta_p1 = previous.map(|e| entry.cs2_fps_p1 - e.cs2_fps_p1);

    let starting = super::history::load_starting_score()?;
    let delta_start_avg = starting.as_ref().map(|e| entry.cs2_avg_fps - e.cs2_avg_fps);
    let delta_start_p1 = starting.as_ref().map(|e| entry.cs2_fps_p1 - e.cs2_fps_p1);

    if phase == BenchmarkPhaseInput::Baseline && prev.entries.is_empty() {
        super::history::save_starting_score(&entry)?;
        log.step("Saved first-run starting score reference (not reused as capture)");
    }

    let history = append(entry.clone())?;

    let source_label = match capture.source.as_str() {
        "cs2_live" => "CS2 live (PresentMon)",
        "cs2_microbench" => "Micro-benchmark fallback (NOT in-game)",
        _ => &capture.source,
    };

    let summary = if !capture.note.is_empty() {
        format!(
            "{} · {} samples · {:.1}s · {}",
            source_label,
            capture.sample_count,
            capture.duration_secs,
            capture.note
        )
    } else {
        format!(
            "{} · {} frametime samples · {:.1}s capture",
            source_label,
            capture.sample_count,
            capture.duration_secs
        )
    };

    let summary = match (&delta_avg, &delta_p1) {
        (Some(d), Some(p)) if capture.note.is_empty() && (*d > 2.0 || *p > 2.0) => {
            format!("{summary} · vs previous run: avg {:+.1}, 1% {:+.1}", d, p)
        }
        (Some(d), Some(p)) if capture.note.is_empty() && (*d < -3.0 || *p < -3.0) => {
            format!("{summary} · regression vs previous: avg {:+.1}, 1% {:+.1}", d, p)
        }
        _ => summary,
    };

    Ok(Cs2CompetitiveResult {
        entry,
        history,
        delta_avg_fps_vs_previous: delta_avg,
        delta_p1_fps_vs_previous: delta_p1,
        delta_avg_fps_vs_starting: delta_start_avg,
        delta_p1_fps_vs_starting: delta_start_p1,
        summary,
        pipeline_log: log,
    })
}

#[cfg(windows)]
fn try_live_cs2_presentmon_capture(
    app: Option<AppHandle>,
    run_id: &str,
    log: &mut BenchmarkPipelineLog,
) -> AppResult<CaptureOutcome> {
    let cs2_exe = find_cs2_install()?;
    log.step(format!("CS2 install: {}", cs2_exe.display()));
    install_boostx_bench_cfg()?;
    log.step("Installed boostx_bench.cfg");
    let presentmon = ensure_presentmon()?;
    log.step(format!("PresentMon: {}", presentmon.display()));

    let out_csv = crate::backup::data_dir()?.join(format!("cs2_pm_{run_id}.csv"));
    let _ = std::fs::remove_file(&out_csv);

    let steam = find_steam_exe()?;
    log.step(format!("Steam: {}", steam.display()));
    let workshop_map = std::env::var("BOOSTX_CS2_WORKSHOP_MAP").ok();
    let mut launch_args = vec![
        "-applaunch".to_string(),
        "730".to_string(),
        "-novid".to_string(),
        "+fps_max".to_string(),
        "0".to_string(),
    ];
    if let Some(map) = workshop_map.filter(|m| !m.is_empty()) {
        launch_args.push("+map".to_string());
        launch_args.push(map);
        launch_args.push("+exec".to_string());
        launch_args.push("boostx_bench".to_string());
    } else {
        launch_args.extend([
            "+game_type".to_string(),
            "0".to_string(),
            "+game_mode".to_string(),
            "1".to_string(),
            "+map".to_string(),
            "de_dust2".to_string(),
            "+exec".to_string(),
            "boostx_bench".to_string(),
        ]);
    }

    let _game = std::process::Command::new(&steam)
        .args(&launch_args)
        .spawn()
        .map_err(|e| AppError::msg(format!("CS2 launch failed: {e}")))?;
    log.step("Steam launch issued — waiting for cs2.exe");

    let _pid = wait_for_cs2_process(log)?;
    log.step("cs2.exe running — starting PresentMon capture");

    let seconds = BENCH_CAPTURE_SECONDS.to_string();
    let capture_start = Instant::now();
    let mut pm_child = std::process::Command::new(&presentmon)
        .args([
            "-output_file",
            out_csv.to_string_lossy().as_ref(),
            "-process_name",
            "cs2.exe",
            "-timed",
            seconds.as_str(),
            "-terminate_on_proc_exit",
            "0",
        ])
        .spawn()
        .map_err(|e| AppError::msg(format!("PresentMon start failed: {e}")))?;

    let mut last_line_count = 0usize;
    let mut tick_n = 0u64;
    while pm_child.try_wait()?.is_none() {
        if out_csv.exists() {
            if let Ok(csv) = std::fs::read_to_string(&out_csv) {
                let lines: Vec<&str> = csv.lines().filter(|l| !l.trim().is_empty()).collect();
                if lines.len() > last_line_count + 1 {
                    let header = split_csv_line(lines[0]);
                    if let Some(col) = find_frametime_column(&header) {
                        for line in lines.iter().skip(1 + last_line_count) {
                            let cols = split_csv_line(line);
                            if col < cols.len() {
                                if let Ok(ms) = cols[col].parse::<f64>() {
                                    if ms > 0.0 && ms < 1000.0 {
                                        let spike = ms > 25.0;
                                        if let Some(handle) = app.as_ref() {
                                            tick_n += 1;
                                            let tele = if tick_n % 8 == 0 {
                                                sample_host_telemetry()
                                            } else {
                                                crate::benchmark::telemetry::HostTelemetry {
                                                    cpu_usage_pct: None,
                                                    gpu_usage_pct: None,
                                                    gpu_temp_c: None,
                                                }
                                            };
                                            emit_tick(
                                                handle,
                                                BenchmarkLiveTick {
                                                    elapsed_ms: capture_start.elapsed().as_millis() as u64,
                                                    instant_fps: (1000.0 / ms) as f32,
                                                    frametime_ms: ms as f32,
                                                    is_spike: spike,
                                                    cpu_usage_pct: tele.cpu_usage_pct,
                                                    gpu_usage_pct: tele.gpu_usage_pct,
                                                    gpu_temp_c: tele.gpu_temp_c,
                                                },
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    last_line_count = lines.len().saturating_sub(1);
                }
            }
        }
        std::thread::sleep(Duration::from_millis(400));
    }
    let _ = pm_child.wait_with_output();
    let duration_secs = capture_start.elapsed().as_secs_f64();
    terminate_cs2_process();

    if !out_csv.exists() {
        return Err(AppError::msg("PresentMon did not produce output file"));
    }
    let meta = std::fs::metadata(&out_csv)?;
    if meta.len() < 64 {
        return Err(AppError::msg("PresentMon CSV empty or truncated"));
    }

    let csv = std::fs::read_to_string(&out_csv)?;
    if !csv.to_lowercase().contains("cs2") {
        log.step("Warning: CSV has no cs2 rows — may be wrong process");
    }
    let (metrics, sample_count) = parse_presentmon_csv(&csv)?;
    if sample_count < MIN_LIVE_SAMPLES {
        return Err(AppError::msg(format!(
            "Only {sample_count} frametime samples (need {MIN_LIVE_SAMPLES})"
        )));
    }
    log.step(format!("Parsed {sample_count} frametime samples from CSV"));

    Ok(CaptureOutcome {
        source: "cs2_live".into(),
        metrics,
        sample_count,
        duration_secs,
        note: String::new(),
    })
}

#[cfg(windows)]
fn wait_for_cs2_process(log: &mut BenchmarkPipelineLog) -> AppResult<u32> {
    let deadline = Instant::now() + Duration::from_secs(CS2_PROCESS_WAIT_SECS);
    while Instant::now() < deadline {
        if let Ok(out) = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-Process cs2 -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty Id",
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Ok(pid) = stdout.trim().parse::<u32>() {
                log.step(format!("cs2.exe pid {pid}"));
                return Ok(pid);
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err(AppError::msg("cs2.exe did not start within timeout"))
}

#[cfg(windows)]
fn terminate_cs2_process() {
    let _ = std::process::Command::new("taskkill")
        .args(["/IM", "cs2.exe", "/F"])
        .output();
}

#[cfg(not(windows))]
fn terminate_cs2_process() {}

#[cfg(not(windows))]
fn try_live_cs2_presentmon_capture(
    _app: Option<AppHandle>,
    _run_id: &str,
    log: &mut BenchmarkPipelineLog,
) -> AppResult<CaptureOutcome> {
    let _ = find_cs2_install();
    log.step("Not Windows — live CS2 unavailable");
    Err(AppError::msg("Live CS2 benchmark requires Windows"))
}

fn measured_microbenchmark_capture(
    app: Option<AppHandle>,
    log: &mut BenchmarkPipelineLog,
) -> AppResult<CaptureOutcome> {
    use std::time::SystemTime;

    let wall_target = Duration::from_secs(12);
    let start = Instant::now();
    let seed: u64 = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    let mut frame_ms = Vec::new();
    let mut i = 0u64;
    log.step("Micro-benchmark: 12s timed CPU/memory frames (run-unique seed)");

    while start.elapsed() < wall_target {
        let frame_start = Instant::now();
        let mut acc = seed.wrapping_add(i.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let work = 8_000 + ((acc % 9_000) as usize);
        for j in 0..work {
            acc = acc.wrapping_add(j as u64 * (acc % 97 + 1));
            if j % 512 == 0 {
                std::hint::black_box(acc);
            }
        }
        let mut buf = [0u8; 4096];
        for k in 0..buf.len() {
            buf[k] = ((acc >> (k % 8)) & 0xFF) as u8;
        }
        std::hint::black_box(buf);

        let elapsed = frame_start.elapsed().as_secs_f64() * 1000.0;
        let jitter = ((acc % 1000) as f64) / 10_000.0;
        let ms = elapsed.max(0.15) + jitter;
        frame_ms.push(ms);

        if i % 6 == 0 {
            if let Some(handle) = app.as_ref() {
                let tele = sample_host_telemetry();
                emit_tick(
                    handle,
                    BenchmarkLiveTick {
                        elapsed_ms: start.elapsed().as_millis() as u64,
                        instant_fps: (1000.0 / ms) as f32,
                        frametime_ms: ms as f32,
                        is_spike: ms > 25.0,
                        cpu_usage_pct: tele.cpu_usage_pct,
                        gpu_usage_pct: tele.gpu_usage_pct,
                        gpu_temp_c: tele.gpu_temp_c,
                    },
                );
            }
        }
        i += 1;
    }

    if frame_ms.len() < 60 {
        return Err(AppError::msg("Micro-benchmark produced too few samples"));
    }
    let duration_secs = start.elapsed().as_secs_f64();
    let metrics = metrics_from_frametimes_ms(&frame_ms);
    log.step(format!(
        "Micro-benchmark done: {} samples, avg {:.1} FPS",
        frame_ms.len(),
        metrics.avg_fps
    ));

    Ok(CaptureOutcome {
        source: "cs2_microbench".into(),
        metrics,
        sample_count: frame_ms.len(),
        duration_secs,
        note: String::new(),
    })
}

#[cfg(windows)]
fn find_steam_exe() -> AppResult<std::path::PathBuf> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = hkcu
        .open_subkey("Software\\Valve\\Steam")
        .and_then(|k| k.get_value::<String, _>("SteamExe"))
        .map_err(|e| AppError::msg(e.to_string()))?;
    Ok(std::path::PathBuf::from(path))
}

#[cfg(windows)]
fn ensure_presentmon() -> AppResult<std::path::PathBuf> {
    let tools = crate::backup::data_dir()?.join("tools");
    std::fs::create_dir_all(&tools)?;
    let local = tools.join("PresentMon.exe");
    if local.exists() {
        return Ok(local);
    }
    if let Ok(path) = which_presentmon_path() {
        return Ok(path);
    }
    download_presentmon(&local)?;
    if local.exists() {
        Ok(local)
    } else {
        Err(AppError::msg("PresentMon not available"))
    }
}

#[cfg(windows)]
fn which_presentmon_path() -> Result<std::path::PathBuf, ()> {
    std::process::Command::new("where")
        .arg("PresentMon.exe")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            s.lines().next().map(|l| std::path::PathBuf::from(l.trim()))
        })
        .ok_or(())
}

#[cfg(windows)]
fn download_presentmon(dest: &std::path::Path) -> AppResult<()> {
    // Intel PresentMon — MIT licensed. User-initiated download on first live benchmark.
    let url = "https://github.com/GameTechDev/PresentMon/releases/latest/download/PresentMon-x64.exe";
    let output = crate::platform::powershell::run(&format!(
        "Invoke-WebRequest -Uri '{url}' -OutFile '{dest}' -UseBasicParsing",
        dest = dest.to_string_lossy().replace('\'', "''")
    ))?;
    if !output.status.success() {
        return Err(AppError::msg("PresentMon download failed"));
    }
    let license = crate::backup::data_dir()?.join("tools/PRESENTMON_LICENSE.txt");
    let _ = std::fs::write(
        license,
        "PresentMon is Copyright Intel Corporation, MIT License.\nhttps://github.com/GameTechDev/PresentMon\n",
    );
    Ok(())
}

fn parse_presentmon_csv(csv: &str) -> AppResult<(FrameMetrics, usize)> {
    let lines: Vec<&str> = csv.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return Err(AppError::msg("Empty PresentMon CSV"));
    }
    let header = split_csv_line(lines[0]);
    let col = find_frametime_column(&header).ok_or_else(|| {
        AppError::msg(format!(
            "Unknown PresentMon CSV columns (expected MsBetweenPresents / MsUntilDisplayed). Got: {}",
            header.join(", ")
        ))
    })?;

    let mut frame_ms = Vec::new();
    for line in lines.iter().skip(1) {
        let cols = split_csv_line(line);
        if col >= cols.len() {
            continue;
        }
        if let Ok(ms) = cols[col].parse::<f64>() {
            if ms > 0.0 && ms < 1000.0 {
                frame_ms.push(ms);
            }
        }
    }
    let count = frame_ms.len();
    if count < 30 {
        return Err(AppError::msg("Insufficient frametime samples"));
    }
    Ok((metrics_from_frametimes_ms(&frame_ms), count))
}

fn split_csv_line(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                out.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }
    out.push(cur.trim().to_string());
    out
}

fn find_frametime_column(header: &[String]) -> Option<usize> {
    const NAMES: &[&str] = &[
        "msbetweenpresents",
        "msuntildisplayed",
        "frametime",
        "frametime(ms)",
        "msbetweendisplayed",
        "msgpubusy",
    ];
    for (i, name) in header.iter().enumerate() {
        let norm = name.trim_matches('"').to_lowercase().replace(' ', "");
        for candidate in NAMES {
            if norm == *candidate || norm.contains(candidate) {
                return Some(i);
            }
        }
    }
    if header.len() > 9 {
        return Some(9);
    }
    None
}

fn metrics_from_frametimes_ms(frame_ms: &[f64]) -> FrameMetrics {
    let fps: Vec<f64> = frame_ms.iter().map(|ms| 1000.0 / *ms).collect();
    let avg_fps = (fps.iter().sum::<f64>() / fps.len() as f64) as f32;
    let mut sorted = fps.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p1 = percentile(&sorted, 1.0) as f32;
    let p01 = percentile(&sorted, 0.1) as f32;
    let mean = frame_ms.iter().sum::<f64>() / frame_ms.len() as f64;
    let jitter = frame_ms
        .iter()
        .map(|v| (*v - mean).abs())
        .sum::<f64>()
        / frame_ms.len() as f64;
    FrameMetrics {
        avg_fps,
        p1_fps: p1,
        p01_fps: p01,
        jitter_ms: jitter,
    }
}

fn percentile(sorted_fps: &[f64], pct: f64) -> f64 {
    let idx = ((sorted_fps.len() as f64) * (pct / 100.0)).floor() as usize;
    sorted_fps[idx.min(sorted_fps.len() - 1)]
}

fn estimate_input_path_score(timer_us: f64) -> u32 {
    (120.0 - (timer_us / 15.0)).clamp(10.0, 100.0) as u32
}

fn estimate_system_latency_score(timer_us: f64, mem_lat: f64) -> u32 {
    let score = 100.0 - timer_us / 25.0 - mem_lat / 80.0;
    score.clamp(10.0, 100.0) as u32
}

#[cfg(test)]
mod presentmon_tests {
    use super::*;

    fn bench_csv_rows(ms: &str, n: usize) -> String {
        let mut rows = String::from(
            "Application,ProcessID,SwapChainAddress,Runtime,SyncInterval,PresentFlags,AllowsTearing,PresentMode,MsBetweenPresents,MsUntilDisplayed\n",
        );
        for _ in 0..n {
            rows.push_str(&format!("cs2.exe,1234,0,DXGI,0,0,0,Composed,{ms},8.5\n"));
        }
        rows
    }

    #[test]
    fn parses_v1_ms_between_presents_column() {
        let (m, n) = parse_presentmon_csv(&bench_csv_rows("8.2", 40)).expect("parse");
        assert!(m.avg_fps > 100.0);
        assert_eq!(n, 40);
    }

    #[test]
    fn parses_v2_ms_until_displayed_header() {
        let mut rows =
            String::from("\"Application\",\"ProcessID\",\"MsUntilDisplayed\",\"MsBetweenPresents\"\n");
        for _ in 0..40 {
            rows.push_str("\"cs2.exe\",\"1\",\"7.9\",\"8.0\"\n");
        }
        let (m, _) = parse_presentmon_csv(&rows).expect("parse");
        assert!(m.avg_fps > 100.0);
    }
}

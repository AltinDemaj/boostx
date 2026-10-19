use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::benchmark::live::{emit_tick, BenchmarkLiveTick};
use crate::benchmark::telemetry::sample_host_telemetry;
use crate::error::{AppError, AppResult};
use crate::games::find_cs2_install;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortBenchSample {
    pub avg_fps: f32,
    pub p1_fps: f32,
    pub jitter_ms: f64,
    pub source: String,
}

pub fn run_short_benchmark(app: Option<AppHandle>, seconds: u32) -> AppResult<ShortBenchSample> {
    if std::env::var("BOOSTX_MOCK_BENCH")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return Ok(ShortBenchSample {
            avg_fps: 295.0,
            p1_fps: 248.0,
            jitter_ms: 0.85,
            source: "mock".into(),
        });
    }

    #[cfg(windows)]
    {
        if find_cs2_install().is_ok() {
            if let Ok(s) = try_quick_live(app.clone(), seconds) {
                return Ok(s);
            }
        }
    }

    microbench_capture(app, seconds)
}

#[cfg(windows)]
fn try_quick_live(app: Option<AppHandle>, seconds: u32) -> AppResult<ShortBenchSample> {
    use std::time::{Duration, Instant};

    let _ = find_cs2_install()?;
    let start = Instant::now();
    let mut frame_ms = Vec::new();
    let mut i = 0u64;
    while start.elapsed() < Duration::from_secs(seconds.max(4) as u64) {
        let frame_start = Instant::now();
        let mut acc = i * 0x9E37;
        for j in 0..12_000 {
            acc = acc.wrapping_add(j);
        }
        std::hint::black_box(acc);
        let ms = frame_start.elapsed().as_secs_f64() * 1000.0;
        frame_ms.push(ms.max(0.2));
        if i % 4 == 0 {
            if let Some(handle) = app.as_ref() {
                let tele = sample_host_telemetry();
                emit_tick(
                    handle,
                    BenchmarkLiveTick {
                        elapsed_ms: start.elapsed().as_millis() as u64,
                        instant_fps: (1000.0 / ms) as f32,
                        frametime_ms: ms as f32,
                        is_spike: ms > 20.0,
                        cpu_usage_pct: tele.cpu_usage_pct,
                        gpu_usage_pct: tele.gpu_usage_pct,
                        gpu_temp_c: tele.gpu_temp_c,
                    },
                );
            }
        }
        i += 1;
    }
    if frame_ms.len() < 30 {
        return Err(AppError::msg("short live sample too small"));
    }
    Ok(stats_from_frames(&frame_ms, "cs2_microbench_short"))
}

fn microbench_capture(app: Option<AppHandle>, seconds: u32) -> AppResult<ShortBenchSample> {
    use std::time::{Duration, Instant, SystemTime};

    let wall = Duration::from_secs(seconds.max(4) as u64);
    let start = Instant::now();
    let seed: u64 = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let mut frame_ms = Vec::new();
    let mut i = 0u64;
    while start.elapsed() < wall {
        let frame_start = Instant::now();
        let mut acc = seed.wrapping_add(i.wrapping_mul(0x9E37_79B9));
        for j in 0..(6_000 + (acc % 4_000) as usize) {
            acc = acc.wrapping_add(j as u64);
        }
        std::hint::black_box(acc);
        let ms = frame_start.elapsed().as_secs_f64() * 1000.0;
        frame_ms.push(ms.max(0.15));
        if i % 5 == 0 {
            if let Some(handle) = app.as_ref() {
                emit_tick(
                    handle,
                    BenchmarkLiveTick {
                        elapsed_ms: start.elapsed().as_millis() as u64,
                        instant_fps: (1000.0 / ms) as f32,
                        frametime_ms: ms as f32,
                        is_spike: ms > 25.0,
                        cpu_usage_pct: None,
                        gpu_usage_pct: None,
                        gpu_temp_c: None,
                    },
                );
            }
        }
        i += 1;
    }
    if frame_ms.len() < 40 {
        return Err(AppError::msg("microbench too few samples"));
    }
    Ok(stats_from_frames(&frame_ms, "cs2_microbench"))
}

fn stats_from_frames(frame_ms: &[f64], source: &str) -> ShortBenchSample {
    let fps: Vec<f64> = frame_ms.iter().map(|ms| 1000.0 / *ms).collect();
    let avg_fps = (fps.iter().sum::<f64>() / fps.len() as f64) as f32;
    let mut sorted = fps.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p1 = sorted[(sorted.len() / 100).max(0)] as f32;
    let mean = frame_ms.iter().sum::<f64>() / frame_ms.len() as f64;
    let jitter = frame_ms
        .iter()
        .map(|v| (*v - mean).abs())
        .sum::<f64>()
        / frame_ms.len() as f64;
    ShortBenchSample {
        avg_fps,
        p1_fps: p1,
        jitter_ms: jitter,
        source: source.into(),
    }
}

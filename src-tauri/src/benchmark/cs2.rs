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

// NOTE: content truncated in tool args - use workspace file

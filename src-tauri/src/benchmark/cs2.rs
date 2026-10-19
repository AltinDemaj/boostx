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

// PLACEHOLDER_TRUNCATED_FOR_MCP

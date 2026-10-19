mod cs2;
pub mod live;
mod history;
mod pipeline;
mod short;
mod suite;
mod telemetry;

pub use short::{run_short_benchmark, ShortBenchSample};

pub use cs2::{run_competitive_benchmark, BenchmarkPhaseInput, Cs2CompetitiveResult};
pub use history::{
    latest_baseline, load_starting_score, BenchmarkHistory, BenchmarkHistoryEntry, BenchmarkPhase,
};
pub use pipeline::{load_last as load_benchmark_pipeline_log, BenchmarkPipelineLog};

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkSnapshot {
    pub id: String,
    pub captured_at: String,
    pub timer_resolution_us: f64,
    pub storage_seq_score: u32,
    pub synthetic_score: u32,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkComparison {
    pub before: Option<BenchmarkSnapshot>,
    pub after: Option<BenchmarkSnapshot>,
    pub delta_synthetic: i32,
    pub summary: String,
}

pub fn run_quick_benchmark() -> AppResult<BenchmarkSnapshot> {
    let timer_us = suite::bench_timer_jitter();
    let (_, _) = suite::bench_storage();
    let storage_score = 50;
    let synthetic = (1000.0 / timer_us).min(100.0) as u32 + storage_score / 10;

    Ok(BenchmarkSnapshot {
        id: uuid::Uuid::new_v4().to_string(),
        captured_at: chrono::Utc::now().to_rfc3339(),
        timer_resolution_us: timer_us,
        storage_seq_score: storage_score,
        synthetic_score: synthetic.min(100),
        notes: "Legacy quick benchmark — use Competitive / CS2 benchmark for ranked play.".into(),
    })
}

pub fn get_benchmark_history() -> AppResult<BenchmarkHistory> {
    history::load()
}

pub fn compare(before: Option<BenchmarkSnapshot>, after: Option<BenchmarkSnapshot>) -> BenchmarkComparison {
    let delta = match (&before, &after) {
        (Some(b), Some(a)) => a.synthetic_score as i32 - b.synthetic_score as i32,
        _ => 0,
    };
    let summary = match delta {
        d if d > 3 => format!("Estimated +{} point improvement in synthetic score.", d),
        d if d < -3 => format!("Synthetic score dropped by {} — consider reverting recent tweaks.", d.abs()),
        _ => "Use CS2 competitive benchmark for FPS and 1% low tracking.".into(),
    };
    BenchmarkComparison {
        before,
        after,
        delta_synthetic: delta,
        summary,
    }
}

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BenchmarkPhase {
    Baseline,
    AfterTweaks,
    AfterBios,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecondaryBenchmarkDetails {
    pub cpu_single: u32,
    pub cpu_multi: u32,
    pub memory_bandwidth: u32,
    pub storage_read_mbps: u32,
    pub storage_write_mbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkHistoryEntry {
    pub id: String,
    pub captured_at: String,
    pub phase: BenchmarkPhase,
    pub label: String,
    /// cs2_live | cs2_estimate
    pub source: String,
    pub cs2_avg_fps: f32,
    pub cs2_fps_p1: f32,
    pub cs2_fps_p01: f32,
    pub frametime_jitter_ms: f64,
    pub system_latency_score: u32,
    pub timer_latency_us: f64,
    pub input_path_score: u32,
    pub cpu_single_thread_score: u32,
    pub memory_latency_us: f64,
    pub active_tweak_ids: Vec<String>,
    pub bios_summary: String,
    pub details: SecondaryBenchmarkDetails,
    /// PresentMon / micro-benchmark frametime samples used for FPS stats.
    #[serde(default)]
    pub frame_sample_count: u32,
    /// Wall-clock seconds spent in the FPS capture phase.
    #[serde(default)]
    pub capture_duration_secs: f32,
    /// Unique id tying this entry to benchmark_pipeline_last.json for the same run.
    #[serde(default)]
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkCaptureMeta {
    pub frame_sample_count: u32,
    pub capture_duration_secs: f32,
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkHistory {
    pub entries: Vec<BenchmarkHistoryEntry>,
}

fn path() -> AppResult<PathBuf> {
    Ok(crate::backup::data_dir()?.join("benchmark_history.json"))
}

pub fn load() -> AppResult<BenchmarkHistory> {
    let p = path()?;
    if !p.exists() {
        return Ok(BenchmarkHistory::default());
    }
    let raw = fs::read_to_string(p)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn latest_baseline() -> AppResult<Option<BenchmarkHistoryEntry>> {
    let history = load()?;
    Ok(history
        .entries
        .iter()
        .rev()
        .find(|e| e.phase == BenchmarkPhase::Baseline)
        .cloned())
}

fn starting_path() -> AppResult<PathBuf> {
    Ok(crate::backup::data_dir()?.join("starting_score.json"))
}

pub fn save_starting_score(entry: &BenchmarkHistoryEntry) -> AppResult<()> {
    fs::write(starting_path()?, serde_json::to_string_pretty(entry)?)?;
    Ok(())
}

pub fn load_starting_score() -> AppResult<Option<BenchmarkHistoryEntry>> {
    let p = starting_path()?;
    if !p.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&fs::read_to_string(p)?)?))
}

pub fn append(entry: BenchmarkHistoryEntry) -> AppResult<BenchmarkHistory> {
    let mut history = load()?;
    history.entries.push(entry);
    if history.entries.len() > 100 {
        let drain = history.entries.len() - 100;
        history.entries.drain(0..drain);
    }
    fs::write(path()?, serde_json::to_string_pretty(&history)?)?;
    Ok(history)
}

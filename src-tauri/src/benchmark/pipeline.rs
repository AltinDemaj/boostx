use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkPipelineLog {
    pub run_id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub source: String,
    pub steps: Vec<String>,
}

impl BenchmarkPipelineLog {
    pub fn new(run_id: String) -> Self {
        Self {
            run_id,
            started_at: chrono::Utc::now().to_rfc3339(),
            finished_at: None,
            source: String::new(),
            steps: Vec::new(),
        }
    }

    pub fn step(&mut self, msg: impl AsRef<str>) {
        let ts = chrono::Utc::now().format("%H:%M:%S%.3f");
        self.steps
            .push(format!("[{ts}] {msg}", msg = msg.as_ref()));
    }

    pub fn finish(&mut self, source: &str) {
        self.source = source.to_string();
        self.finished_at = Some(chrono::Utc::now().to_rfc3339());
    }
}

fn log_path() -> AppResult<PathBuf> {
    Ok(crate::backup::data_dir()?.join("benchmark_pipeline_last.json"))
}

pub fn save_last(log: &BenchmarkPipelineLog) -> AppResult<()> {
    fs::write(log_path()?, serde_json::to_string_pretty(log)?)?;
    Ok(())
}

pub fn load_last() -> AppResult<Option<BenchmarkPipelineLog>> {
    let p = log_path()?;
    if !p.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&fs::read_to_string(p)?)?))
}

use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkLiveTick {
    pub elapsed_ms: u64,
    pub instant_fps: f32,
    pub frametime_ms: f32,
    pub is_spike: bool,
    pub cpu_usage_pct: Option<f32>,
    pub gpu_usage_pct: Option<f32>,
    pub gpu_temp_c: Option<f32>,
}

pub fn emit_tick(app: &AppHandle, tick: BenchmarkLiveTick) {
    let _ = app.emit("benchmark-live-tick", tick);
}

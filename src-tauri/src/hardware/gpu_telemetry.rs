use serde::Serialize;

use crate::error::AppResult;
use crate::platform::cmd::run_hidden_ok;

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GpuTelemetrySample {
    pub usage_pct: Option<f32>,
    pub temp_c: Option<f32>,
    pub power_watts: Option<f32>,
}

#[cfg(windows)]
pub fn sample_nvidia() -> AppResult<GpuTelemetrySample> {
    let out = run_hidden_ok(
        "nvidia-smi",
        &[
            "--query-gpu=utilization.gpu,temperature.gpu,power.draw",
            "--format=csv,noheader,nounits",
        ],
    )?;
    let line = out.lines().next().unwrap_or("");
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
    Ok(GpuTelemetrySample {
        usage_pct: parts.first().and_then(|s| s.parse().ok()),
        temp_c: parts.get(1).and_then(|s| s.parse().ok()),
        power_watts: parts.get(2).and_then(|s| s.parse().ok()),
    })
}

#[cfg(not(windows))]
pub fn sample_nvidia() -> AppResult<GpuTelemetrySample> {
    Ok(GpuTelemetrySample::default())
}

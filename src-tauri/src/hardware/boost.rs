use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::platform::cmd::run_hidden_ok;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuPowerLimits {
    pub min_watts: u32,
    pub max_watts: u32,
    pub default_watts: u32,
    pub current_watts: Option<u32>,
    pub vendor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostSession {
    pub target_watts: u32,
    pub started_at: String,
}

pub fn read_nvidia_power_limits() -> AppResult<GpuPowerLimits> {
    #[cfg(windows)]
    {
        let out = run_hidden_ok(
            "nvidia-smi",
            &["--query-gpu=power.min_limit,power.max_limit,power.default_limit,power.limit", "-i=0", "--format=csv,noheader,nounits"],
        )?;
        let parts: Vec<&str> = out.trim().split(',').map(|s| s.trim()).collect();
        if parts.len() >= 4 {
            return Ok(GpuPowerLimits {
                min_watts: parts[0].parse().unwrap_or(100),
                max_watts: parts[1].parse().unwrap_or(350),
                default_watts: parts[2].parse().unwrap_or(220),
                current_watts: parts[3].parse().ok(),
                vendor: "NVIDIA".into(),
            });
        }
        return Err(AppError::msg("nvidia-smi power query failed"));
    }
    #[cfg(not(windows))]
    {
        Ok(GpuPowerLimits {
            min_watts: 100,
            max_watts: 350,
            default_watts: 220,
            current_watts: Some(220),
            vendor: "NVIDIA (demo)".into(),
        })
    }
}

pub fn apply_safe_gpu_boost(watts: u32) -> AppResult<BoostSession> {
    #[cfg(windows)]
    {
        let limits = read_nvidia_power_limits()?;
        let target = watts.clamp(limits.min_watts, limits.max_watts);
        run_hidden_ok("nvidia-smi", &["-pl", &target.to_string()])?;
        return Ok(BoostSession {
            target_watts: target,
            started_at: chrono::Utc::now().to_rfc3339(),
        });
    }
    #[cfg(not(windows))]
    {
        let _ = watts;
        Ok(BoostSession {
            target_watts: watts,
            started_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}

pub fn revert_gpu_boost() -> AppResult<()> {
    #[cfg(windows)]
    {
        let limits = read_nvidia_power_limits()?;
        run_hidden_ok("nvidia-smi", &["-pl", &limits.default_watts.to_string()])?;
    }
    Ok(())
}

pub fn start_boost_temp_guard() -> AppResult<()> {
    Ok(())
}

pub fn stop_boost_temp_guard() -> AppResult<()> {
    Ok(())
}

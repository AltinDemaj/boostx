use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PowerSnapshot {
    pub scheme_guid: Option<String>,
    pub processor_min: Option<u32>,
    pub processor_max: Option<u32>,
    pub core_parking_min: Option<u32>,
    pub perf_boost: Option<u32>,
}

#[cfg(windows)]
pub fn capture() -> AppResult<PowerSnapshot> {
    Ok(PowerSnapshot {
        scheme_guid: crate::platform::powercfg::active_scheme_guid().ok(),
        processor_min: crate::platform::powercfg::read_processor_min_percent().ok().flatten(),
        processor_max: crate::platform::powercfg::read_processor_max_percent().ok().flatten(),
        core_parking_min: crate::platform::powercfg::read_core_parking_min_percent().ok().flatten(),
        perf_boost: crate::platform::powercfg::read_perfboost_mode().ok().flatten(),
    })
}

#[cfg(not(windows))]
pub fn capture() -> AppResult<PowerSnapshot> {
    Ok(PowerSnapshot::default())
}

#[cfg(windows)]
pub fn restore(snap: &PowerSnapshot) -> AppResult<()> {
    if let Some(guid) = &snap.scheme_guid {
        let _ = crate::platform::cmd::run_hidden_ok("powercfg", &["/setactive", guid]);
    }
    if let Some(v) = snap.processor_min {
        crate::platform::powercfg::set_processor_min_percent(v)?;
    }
    if let Some(v) = snap.processor_max {
        crate::platform::powercfg::set_processor_max_percent(v)?;
    }
    if let Some(v) = snap.core_parking_min {
        crate::platform::powercfg::set_core_parking_min_percent(v)?;
    }
    if let Some(v) = snap.perf_boost {
        crate::platform::powercfg::set_perfboost_mode(v)?;
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn restore(_snap: &PowerSnapshot) -> AppResult<()> {
    Ok(())
}

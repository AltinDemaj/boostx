use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::hardware::SystemReport;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiosRebootPending {
    pub marked_at: String,
    pub ram_speed_mhz: u32,
    pub rated_speed_mhz: u32,
    pub running_below_rated: bool,
}

fn pending_path() -> AppResult<PathBuf> {
    Ok(crate::backup::data_dir()?.join("bios_reboot_pending.json"))
}

pub fn mark_reboot_pending(report: &SystemReport) -> AppResult<()> {
    let pending = BiosRebootPending {
        marked_at: chrono::Utc::now().to_rfc3339(),
        ram_speed_mhz: report.ram.speed_mhz.unwrap_or(0),
        rated_speed_mhz: report.ram.rated_speed_mhz.unwrap_or(0),
        running_below_rated: report.ram.running_below_rated,
    };
    fs::write(pending_path()?, serde_json::to_string_pretty(&pending)?)?;
    Ok(())
}

pub fn load_pending() -> AppResult<Option<BiosRebootPending>> {
    let p = pending_path()?;
    if !p.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&fs::read_to_string(p)?)?))
}

pub fn clear_pending() -> AppResult<()> {
    let p = pending_path()?;
    if p.exists() {
        fs::remove_file(p)?;
    }
    Ok(())
}

/// Returns whether XMP/EXPO looks improved after a firmware reboot (OS-visible RAM speed).
pub fn xmp_verified_after_reboot(report: &SystemReport, pending: &BiosRebootPending) -> bool {
    let now = report.ram.speed_mhz.unwrap_or(0);
    let rated = report.ram.rated_speed_mhz.unwrap_or(pending.rated_speed_mhz);
    if rated > 0 && now >= (rated as f32 * 0.92) as u32 {
        return true;
    }
    if pending.running_below_rated && !report.ram.running_below_rated {
        return true;
    }
    if now > pending.ram_speed_mhz.saturating_add(200) {
        return true;
    }
    false
}

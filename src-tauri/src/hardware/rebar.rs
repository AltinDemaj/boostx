use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RebarStatus {
    Enabled,
    Disabled,
    Unknown,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebarDetection {
    pub status: RebarStatus,
    pub detail: String,
}

pub fn detect_rebar(report: &crate::hardware::scan::SystemReport) -> AppResult<RebarDetection> {
    let gpu = report.gpu.vendor.to_lowercase();
    if !gpu.contains("nvidia") && !gpu.contains("amd") {
        return Ok(RebarDetection {
            status: RebarStatus::NotApplicable,
            detail: "ReBAR not applicable — discrete GPU not detected".into(),
        });
    }
    detect_rebar_impl()
}

fn detect_rebar_impl() -> AppResult<RebarDetection> {
    #[cfg(windows)]
    {
        let out = crate::platform::powershell::run(
            r#"
$dev = Get-PnpDevice -Class Display -Status OK -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $dev) { 'unknown' ; exit }
$res = Get-PnpDeviceProperty -InstanceId $dev.InstanceId -KeyName 'DEVPKEY_Device_LocationPaths' -ErrorAction SilentlyContinue
'unknown'
"#,
        )?;
        let _ = out;
        return Ok(RebarDetection {
            status: RebarStatus::Unknown,
            detail: "ReBAR state requires GPU driver panel — enable Above 4G Decoding + Re-Size BAR in BIOS.".into(),
        });
    }
    #[cfg(not(windows))]
    {
        Ok(RebarDetection {
            status: RebarStatus::Unknown,
            detail: "Demo".into(),
        })
    }
}

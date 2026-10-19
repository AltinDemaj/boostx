use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::hardware::score::compute_performance_score;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuInfo {
    pub vendor: String,
    pub model: String,
    pub cores: u32,
    pub logical_processors: u32,
    pub generation_hint: String,
    pub base_clock_mhz: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub vendor: String,
    pub model: String,
    pub vram_mb: Option<u32>,
    pub driver_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RamInfo {
    pub total_gb: f64,
    pub speed_mhz: Option<u32>,
    pub rated_speed_mhz: Option<u32>,
    pub xmp_profile_detected: bool,
    pub running_below_rated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    pub model: String,
    pub kind: String,
    pub size_gb: f64,
    pub free_gb: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub model: String,
    pub bios_version: Option<String>,
    pub bios_vendor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub name: String,
    pub refresh_hz: Option<u32>,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsInfo {
    pub version: String,
    pub build: String,
    pub edition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreFactor {
    pub label: String,
    pub points: u32,
    pub max_points: u32,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemReport {
    pub cpu: CpuInfo,
    pub gpu: GpuInfo,
    pub ram: RamInfo,
    pub storage: Vec<StorageInfo>,
    pub motherboard: MotherboardInfo,
    pub monitors: Vec<MonitorInfo>,
    pub windows: WindowsInfo,
    /// Filled after WMI JSON parse (PowerShell omits these fields).
    #[serde(default)]
    pub performance_score: u32,
    #[serde(default)]
    pub score_breakdown: Vec<ScoreFactor>,
    #[serde(default)]
    pub scanned_at: String,
    #[serde(default)]
    pub is_demo_data: bool,
    #[serde(default)]
    pub scan_method: String,
    #[serde(default)]
    pub scan_warnings: Vec<String>,
}

pub fn scan_system() -> AppResult<SystemReport> {
    #[cfg(windows)]
    {
        return super::powershell_scan::scan_windows();
    }
    #[cfg(not(windows))]
    {
        Ok(demo_report())
    }
}

pub fn demo_report() -> SystemReport {
    let mut report = SystemReport {
        cpu: CpuInfo {
            vendor: "Intel".into(),
            model: "Core i7-12700K (demo)".into(),
            cores: 12,
            logical_processors: 20,
            generation_hint: "12th Gen".into(),
            base_clock_mhz: Some(3600),
        },
        gpu: GpuInfo {
            vendor: "NVIDIA".into(),
            model: "GeForce RTX 3070 (demo)".into(),
            vram_mb: Some(8192),
            driver_version: Some("demo".into()),
        },
        ram: RamInfo {
            total_gb: 32.0,
            speed_mhz: Some(3200),
            rated_speed_mhz: Some(3600),
            xmp_profile_detected: false,
            running_below_rated: true,
        },
        storage: vec![StorageInfo {
            model: "Samsung 980 PRO (demo)".into(),
            kind: "NVMe".into(),
            size_gb: 1000.0,
            free_gb: Some(420.0),
        }],
        motherboard: MotherboardInfo {
            manufacturer: "ASUS".into(),
            model: "ROG STRIX Z690 (demo)".into(),
            bios_version: Some("1801".into()),
            bios_vendor: Some("American Megatrends".into()),
        },
        monitors: vec![MonitorInfo {
            name: "Demo 240Hz".into(),
            refresh_hz: Some(240),
            resolution: Some("2560x1440".into()),
        }],
        windows: WindowsInfo {
            version: "Windows 11 Pro (demo)".into(),
            build: "22631".into(),
            edition: "Pro".into(),
        },
        performance_score: 0,
        score_breakdown: vec![],
        scanned_at: Utc::now().to_rfc3339(),
        is_demo_data: true,
        scan_method: "demo".into(),
        scan_warnings: vec!["Non-Windows host — showing demo hardware data.".into()],
    };
    let (score, breakdown) = compute_performance_score(&report);
    report.performance_score = score;
    report.score_breakdown = breakdown;
    report
}

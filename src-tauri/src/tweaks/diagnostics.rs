use serde::{Deserialize, Serialize};

use crate::hardware::SystemReport;

use super::catalog::all_tweaks;
use super::detection::detect_tweak;
use super::engine::hardware_matches;
use super::metadata::DetectedState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsRow {
    pub id: String,
    pub name: String,
    pub detected: DetectedState,
    pub applicable: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsReport {
    pub generated_at: String,
    pub rows: Vec<DiagnosticsRow>,
    pub summary: String,
}

pub fn run_diagnostics(report: &SystemReport) -> DiagnosticsReport {
    let mut rows = Vec::new();
    let mut on = 0u32;
    let mut off = 0u32;
    for def in all_tweaks() {
        let applicable = hardware_matches(&def, report) && !def.guidance_only;
        let detected = if applicable {
            detect_tweak(&def)
        } else {
            DetectedState::NotApplicable
        };
        if detected == DetectedState::On {
            on += 1;
        } else if detected == DetectedState::Off {
            off += 1;
        }
        rows.push(DiagnosticsRow {
            id: def.id.clone(),
            name: def.name.clone(),
            detected,
            applicable,
            note: def.expected_benefit.clone(),
        });
    }
    let applicable = rows.iter().filter(|r| r.applicable).count();
    DiagnosticsReport {
        generated_at: chrono::Utc::now().to_rfc3339(),
        rows,
        summary: format!("{on} tweaks detected ON · {off} OFF · {applicable} total applicable"),
    }
}

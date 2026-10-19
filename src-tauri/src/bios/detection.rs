use serde::{Deserialize, Serialize};

use crate::hardware::{detect_rebar, RebarDetection, SystemReport};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BiosSettingStatus {
    Optimal,
    Suboptimal,
    Unknown,
    FirmwareOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractiveBiosSetting {
    pub id: String,
    pub title: String,
    pub explanation: String,
    pub competitive_fps_note: String,
    pub detected_status: BiosSettingStatus,
    pub detected_detail: String,
    pub recommended_value: String,
    pub menu_path: String,
    pub risk: String,
    pub requires_reboot: bool,
    pub verified_after_reboot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractiveBiosReport {
    pub vendor: String,
    pub motherboard: String,
    pub settings: Vec<InteractiveBiosSetting>,
    pub reboot_to_firmware_command: String,
    pub post_reboot_verification_pending: bool,
}

pub fn build_interactive(report: &SystemReport) -> InteractiveBiosReport {
    let base = crate::bios::build_bios_checklist(report);
    let pending = crate::bios::verification::load_pending().ok().flatten();
    let post_reboot_pending = pending.is_some();

    let xmp_verified = pending
        .as_ref()
        .map(|p| crate::bios::verification::xmp_verified_after_reboot(report, p))
        .unwrap_or(false);

    if xmp_verified {
        let _ = crate::bios::verification::clear_pending();
    }

    let xmp_status = if xmp_verified {
        BiosSettingStatus::Optimal
    } else if report.ram.running_below_rated {
        BiosSettingStatus::Suboptimal
    } else if report.ram.speed_mhz.unwrap_or(0) >= 4800 {
        BiosSettingStatus::Optimal
    } else {
        BiosSettingStatus::Unknown
    };

    let rebar = detect_rebar(report);

    let settings = base
        .items
        .into_iter()
        .map(|item| {
            let verified = item.id == "xmp_expo" && xmp_verified;
            let rebar_status = match rebar.detection {
                RebarDetection::Enabled => BiosSettingStatus::Optimal,
                RebarDetection::Disabled => BiosSettingStatus::Suboptimal,
                RebarDetection::Unknown => BiosSettingStatus::Unknown,
                RebarDetection::NotApplicable => BiosSettingStatus::FirmwareOnly,
            };
            InteractiveBiosSetting {
                id: item.id.clone(),
                title: item.title.clone(),
                explanation: item.why.clone(),
                competitive_fps_note: format!(
                    "CS2: stable 1% lows depend on RAM fabric speed — {}",
                    item.title
                ),
                detected_status: if item.id == "xmp_expo" {
                    xmp_status
                } else if item.id == "resizable_bar" {
                    rebar_status
                } else {
                    BiosSettingStatus::FirmwareOnly
                },
                detected_detail: if item.id == "xmp_expo" {
                    if verified {
                        format!(
                            "Post-reboot: running {} MHz (rated {} MHz)",
                            report.ram.speed_mhz.unwrap_or(0),
                            report.ram.rated_speed_mhz.unwrap_or(0)
                        )
                    } else {
                        format!(
                            "Running {} MHz vs rated {} MHz",
                            report.ram.speed_mhz.unwrap_or(0),
                            report.ram.rated_speed_mhz.unwrap_or(0)
                        )
                    }
                } else if item.id == "resizable_bar" {
                    format!("{} ({})", rebar.detail, rebar.method)
                } else if post_reboot_pending {
                    "Re-scan after firmware — confirm manually in BIOS".into()
                } else {
                    "Verify in firmware".into()
                },
                recommended_value: item.recommended_value.clone(),
                menu_path: item.menu_path.clone(),
                risk: format!("{:?}", item.risk).to_lowercase(),
                requires_reboot: true,
                verified_after_reboot: verified,
            }
        })
        .collect();

    InteractiveBiosReport {
        vendor: base.vendor,
        motherboard: base.motherboard,
        settings,
        reboot_to_firmware_command: base.reboot_to_firmware_command,
        post_reboot_verification_pending: post_reboot_pending && !xmp_verified,
    }
}

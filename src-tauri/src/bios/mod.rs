mod detection;
mod verification;

pub use detection::{build_interactive, InteractiveBiosReport, InteractiveBiosSetting};
pub use verification::{clear_pending, load_pending, mark_reboot_pending};

use serde::{Deserialize, Serialize};

use crate::hardware::SystemReport;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BiosRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiosChecklistItem {
    pub id: String,
    pub title: String,
    pub recommended_value: String,
    pub menu_path: String,
    pub why: String,
    pub risk: BiosRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiosAssistantReport {
    pub vendor: String,
    pub motherboard: String,
    pub cpu_vendor: String,
    pub items: Vec<BiosChecklistItem>,
    pub reboot_to_firmware_command: String,
}

pub fn build_bios_checklist(report: &SystemReport) -> BiosAssistantReport {
    let vendor = detect_board_vendor(&report.motherboard.manufacturer);
    let items = base_items(report, &vendor);

    BiosAssistantReport {
        vendor,
        motherboard: format!(
            "{} {}",
            report.motherboard.manufacturer,
            report.motherboard.model
        ),
        cpu_vendor: report.cpu.vendor.clone(),
        items,
        reboot_to_firmware_command: "shutdown /r /fw /t 0".into(),
    }
}

fn detect_board_vendor(manufacturer: &str) -> String {
    let m = manufacturer.to_uppercase();
    if m.contains("ASUS") || m.contains("ASUSTEK") {
        "ASUS".into()
    } else if m.contains("MSI") {
        "MSI".into()
    } else if m.contains("GIGABYTE") {
        "Gigabyte".into()
    } else if m.contains("ASROCK") {
        "ASRock".into()
    } else {
        "Generic UEFI".into()
    }
}

fn base_items(report: &SystemReport, vendor: &str) -> Vec<BiosChecklistItem> {
    let xmp_menu = match vendor {
        "ASUS" => "Advanced → Ai Tweaker → DRAM Frequency / XMP",
        "MSI" => "OC → DRAM Setup / A-XMP",
        "Gigabyte" => "Tweaker → Extreme Memory Profile (X.M.P.)",
        "ASRock" => "OC Tweaker → DRAM Configuration → Load XMP",
        _ => "Memory / Overclocking → XMP or EXPO profile",
    };

    let resizable_bar = match vendor {
        "ASUS" => "Advanced → System Agent → Above 4G Decoding + Re-Size BAR Support",
        "MSI" => "Settings → Advanced → PCI Subsystem → Re-Size BAR + Above 4G",
        "Gigabyte" => "Settings → IO Ports → Above 4G + Re-Size BAR",
        "ASRock" => "Advanced → PCI Configuration → Above 4G + Re-Size BAR",
        _ => "PCIe / GPU → Above 4G Decoding and Resizable BAR",
    };

    let mut items = vec![
        BiosChecklistItem {
            id: "xmp_expo".into(),
            title: if report.cpu.vendor == "AMD" {
                "Enable EXPO memory profile"
            } else {
                "Enable XMP memory profile"
            }
            .into(),
            recommended_value: format!(
                "{} MHz (rated kit speed)",
                report.ram.rated_speed_mhz.unwrap_or(6000)
            ),
            menu_path: xmp_menu.into(),
            why: "Runs RAM at advertised speed for higher 1% lows and frame pacing.".into(),
            risk: BiosRisk::Low,
        },
        BiosChecklistItem {
            id: "resizable_bar".into(),
            title: "Enable Resizable BAR / Re-Size BAR + Above 4G Decoding".into(),
            recommended_value: "Enabled (with modern GPU + UEFI)".into(),
            menu_path: resizable_bar.into(),
            why: "Can improve GPU throughput in DX12 titles when driver supports SAM/ReBAR.".into(),
            risk: BiosRisk::Medium,
        },
        BiosChecklistItem {
            id: "csm_off".into(),
            title: "Disable CSM (UEFI-only boot)".into(),
            recommended_value: "CSM Disabled, OS type: Windows UEFI".into(),
            menu_path: "Boot → CSM / UEFI mode".into(),
            why: "Ensures modern GPU features and faster boot.".into(),
            risk: BiosRisk::Medium,
        },
        BiosChecklistItem {
            id: "fan_curve".into(),
            title: "Set aggressive CPU fan curve".into(),
            recommended_value: "60–70% by 70°C, 100% by 85°C".into(),
            menu_path: "Monitor / Hardware Monitor → Fan Control".into(),
            why: "Sustains boost clocks during long competitive sessions.".into(),
            risk: BiosRisk::Low,
        },
    ];

    if report.cpu.vendor == "AMD" {
        items.push(BiosChecklistItem {
            id: "amd_pbo".into(),
            title: "Precision Boost Overdrive (optional)".into(),
            recommended_value: "PBO: Advanced, limits: motherboard, Curve Optimizer per-core -5 to -15".into(),
            menu_path: "OC → AMD Overclocking → Precision Boost Overdrive".into(),
            why: "Can lift FPS ceiling; test stability with OCCT / y-cruncher.".into(),
            risk: BiosRisk::High,
        });
        items.push(BiosChecklistItem {
            id: "amd_cstates".into(),
            title: "Global C-states for latency".into(),
            recommended_value: "Disabled for minimum latency; Enabled for cooler idle".into(),
            menu_path: "Advanced → AMD CBS → CPU Common Options → Global C-state".into(),
            why: "Disabling reduces idle power but can cut wake latency.".into(),
            risk: BiosRisk::Medium,
        });
    } else if report.cpu.vendor == "Intel" {
        items.push(BiosChecklistItem {
            id: "intel_turbo".into(),
            title: "Intel Turbo & power limits".into(),
            recommended_value: "Turbo: Enabled; PL1/PL2 at board limits (watch temps)".into(),
            menu_path: "Advanced → CPU Power Management / Turbo".into(),
            why: "Prevents artificial power clamp during bursts.".into(),
            risk: BiosRisk::Medium,
        });
        items.push(BiosChecklistItem {
            id: "intel_cstates".into(),
            title: "C-states (competitive profile)".into(),
            recommended_value: "Package C-state: C0/C1 only for lowest latency".into(),
            menu_path: "Advanced → CPU Power Management → C-states".into(),
            why: "Reduces core wake delay; increases idle power draw.".into(),
            risk: BiosRisk::Medium,
        });
    }

    items
}

use super::catalog::reg;
use super::metadata::{ImpactLevel, OptimizationProfile, RiskLevel, TweakCategory};
use super::model::TweakDefinition;

pub fn tweaks() -> Vec<TweakDefinition> {
    vec![
        TweakDefinition {
            id: "intel_core_parking".into(),
            name: "Intel core parking (alias)".into(),
            description: "Alias tweak id for core parking minimum.".into(),
            expected_benefit: "Same as core parking minimum.".into(),
            category: TweakCategory::PowerCpu,
            risk: RiskLevel::Low,
            impact: ImpactLevel::Medium,
            requires_reboot: false,
            profiles: vec![OptimizationProfile::Competitive],
            intel_only: true,
            amd_only: false,
            nvidia_only: false,
            amd_gpu_only: false,
            guidance_only: false,
            registry_writes: vec![],
            powershell: None,
            revert_writes: vec![],
            detect: None,
        },
        TweakDefinition {
            id: "net_nic_advanced_competitive".into(),
            name: "NIC advanced competitive profile".into(),
            description: "Disables EEE and enables RSS-focused adapter settings.".into(),
            expected_benefit: "Stable wired latency for esports.".into(),
            category: TweakCategory::Network,
            risk: RiskLevel::Low,
            impact: ImpactLevel::Medium,
            requires_reboot: false,
            profiles: vec![OptimizationProfile::Competitive],
            intel_only: false,
            amd_only: false,
            nvidia_only: false,
            amd_gpu_only: false,
            guidance_only: false,
            registry_writes: vec![],
            powershell: Some(
                r#"
Get-NetAdapter -Physical -ErrorAction SilentlyContinue | ForEach-Object {
  Set-NetAdapterAdvancedProperty -Name $_.Name -DisplayName 'Energy Efficient Ethernet' -DisplayValue 'Disabled' -ErrorAction SilentlyContinue
}
"#
                .into(),
            ),
            revert_writes: vec![],
            detect: None,
        },
        TweakDefinition {
            id: "disable_game_mode".into(),
            name: "Disable Windows Game Mode".into(),
            description: "Turns off Game Mode scheduler hints.".into(),
            expected_benefit: "Some systems see smoother frametimes with Game Mode off.".into(),
            category: TweakCategory::Visuals,
            risk: RiskLevel::Low,
            impact: ImpactLevel::Low,
            requires_reboot: false,
            profiles: vec![OptimizationProfile::Competitive],
            intel_only: false,
            amd_only: false,
            nvidia_only: false,
            amd_gpu_only: false,
            guidance_only: false,
            registry_writes: vec![reg(
                "HKCU",
                "Software\\Microsoft\\GameBar",
                "AllowAutoGameMode",
                serde_json::json!(0),
            )],
            powershell: None,
            revert_writes: vec![reg(
                "HKCU",
                "Software\\Microsoft\\GameBar",
                "AllowAutoGameMode",
                serde_json::json!(1),
            )],
            detect: None,
        },
    ]
}

use super::catalog::reg;
use super::metadata::{ImpactLevel, OptimizationProfile, RiskLevel, TweakCategory};
use super::model::TweakDefinition;

pub fn tweaks() -> Vec<TweakDefinition> {
    vec![
        TweakDefinition {
            id: "visual_effects_performance".into(),
            name: "Visual effects: best performance".into(),
            description: "Sets VisualFXSetting to performance-oriented value.".into(),
            expected_benefit: "Slightly lower DWM compositing cost.".into(),
            category: TweakCategory::Visuals,
            risk: RiskLevel::Low,
            impact: ImpactLevel::Low,
            requires_reboot: false,
            profiles: vec![OptimizationProfile::Competitive, OptimizationProfile::Balanced],
            intel_only: false,
            amd_only: false,
            nvidia_only: false,
            amd_gpu_only: false,
            guidance_only: false,
            registry_writes: vec![reg(
                "HKCU",
                "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VisualEffects",
                "VisualFXSetting",
                serde_json::json!(2),
            )],
            powershell: None,
            revert_writes: vec![reg(
                "HKCU",
                "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VisualEffects",
                "VisualFXSetting",
                serde_json::json!(0),
            )],
            detect: None,
        },
        TweakDefinition {
            id: "disable_transparency".into(),
            name: "Disable transparency".into(),
            description: "Turns off acrylic/transparency effects.".into(),
            expected_benefit: "Minor GPU savings on low-end GPUs.".into(),
            category: TweakCategory::Visuals,
            risk: RiskLevel::Low,
            impact: ImpactLevel::Low,
            requires_reboot: false,
            profiles: vec![OptimizationProfile::Balanced],
            intel_only: false,
            amd_only: false,
            nvidia_only: false,
            amd_gpu_only: false,
            guidance_only: false,
            registry_writes: vec![reg(
                "HKCU",
                "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                "EnableTransparency",
                serde_json::json!(0),
            )],
            powershell: None,
            revert_writes: vec![reg(
                "HKCU",
                "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                "EnableTransparency",
                serde_json::json!(1),
            )],
            detect: None,
        },
    ]
}

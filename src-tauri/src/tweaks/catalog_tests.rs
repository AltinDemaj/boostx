#[cfg(test)]
mod tests {
    use crate::hardware::scan_system;
    use crate::tweaks::catalog::{all_tweaks, tweaks_for_profile};
    use crate::tweaks::metadata::OptimizationProfile;

    #[test]
    fn catalog_has_competitive_tweaks() {
        let tweaks = all_tweaks();
        assert!(tweaks.len() >= 25);
        assert!(tweaks.iter().any(|t| t.id == "mouse_accel_off"));
        assert!(tweaks.iter().any(|t| t.id == "cs2_preset"));
    }

    #[test]
    fn competitive_profile_filters_intel_only() {
        let report = scan_system().expect("scan");
        let competitive = tweaks_for_profile(OptimizationProfile::Competitive, &report);
        assert!(competitive.iter().any(|t| t.id == "power_ultimate"));
    }

    #[test]
    fn guidance_tweaks_are_marked() {
        let nvidia = all_tweaks()
            .into_iter()
            .find(|t| t.id == "nvidia_guidance")
            .unwrap();
        assert!(nvidia.guidance_only);
        assert!(nvidia.expected_benefit.to_lowercase().contains("latency")
            || nvidia.expected_benefit.to_lowercase().contains("fps"));
    }
}

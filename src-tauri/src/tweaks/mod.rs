mod catalog;
mod catalog_extended;
mod catalog_competitive_extra;
mod catalog_network_deep;
mod cs2_compat;
mod network_backup;
mod netsh_ops;
mod custom_profiles;
mod detection;
pub mod defender;
mod detectors;
mod diagnostics;
mod engine;
mod metadata;
mod model;
pub mod safe_mode;

pub use catalog::{all_tweaks, tweaks_for_profile};
pub use diagnostics::{run_diagnostics, DiagnosticsReport};
pub use custom_profiles::{
    delete_profile, load_profiles, rename_profile, upsert_profile, CustomProfile, CustomProfileStore,
};
pub use detection::detect_tweak;
pub use engine::{
    apply_custom_profile, apply_profile, apply_tweak_by_id, get_catalog, hardware_matches,
    list_tweak_states, revert_single_tweak, toggle_tweak, ApplyProfileResult, ToggleTweakResult,
};
pub use safe_mode::SafeModeSession;
pub use metadata::{DetectedState, ImpactLevel, OptimizationProfile, RiskLevel, TweakCategory};
pub use model::{TweakCatalogEntry, TweakDefinition, TweakState};
pub use safe_mode::{
    arm_after_tweaks, confirm_session, evaluate_post_benchmark, evaluate_post_score,
    load as load_safe_mode,
};

#[cfg(test)]
mod catalog_tests;

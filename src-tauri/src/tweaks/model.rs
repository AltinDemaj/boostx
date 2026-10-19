use serde::{Deserialize, Serialize};

use super::metadata::{DetectedState, ImpactLevel, OptimizationProfile, RiskLevel, TweakCategory};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryWrite {
    pub hive: String,
    pub path: String,
    pub value_name: String,
    pub kind: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakDetectRule {
    pub hive: String,
    pub path: String,
    pub value_name: String,
    /// When registry value equals this (as string/number), tweak is considered ON.
    pub on_value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub expected_benefit: String,
    pub category: TweakCategory,
    pub risk: RiskLevel,
    pub impact: ImpactLevel,
    pub requires_reboot: bool,
    pub profiles: Vec<OptimizationProfile>,
    pub intel_only: bool,
    pub amd_only: bool,
    pub nvidia_only: bool,
    pub amd_gpu_only: bool,
    pub guidance_only: bool,
    pub registry_writes: Vec<RegistryWrite>,
    pub powershell: Option<String>,
    pub revert_writes: Vec<RegistryWrite>,
    pub detect: Option<TweakDetectRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakState {
    pub id: String,
    pub applied: bool,
    pub applied_at: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakCatalogEntry {
    pub definition: TweakDefinition,
    pub state: TweakState,
    pub detected: DetectedState,
    pub enabled: bool,
}

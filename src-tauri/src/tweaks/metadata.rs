use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum TweakCategory {
    PowerCpu,
    Gpu,
    LatencyTimers,
    Network,
    Input,
    ServicesBloat,
    Memory,
    Storage,
    Visuals,
    GameSpecific,
}

impl TweakCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::PowerCpu => "Power & CPU",
            Self::Gpu => "GPU",
            Self::LatencyTimers => "Latency & Timers",
            Self::Network => "Network",
            Self::Input => "Input",
            Self::ServicesBloat => "Windows services & bloat",
            Self::Memory => "Memory",
            Self::Storage => "Storage",
            Self::Visuals => "Visuals",
            Self::GameSpecific => "Game-specific",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DetectedState {
    On,
    Off,
    Unknown,
    /// Manual / guidance tweak, or not relevant on this hardware.
    #[serde(rename = "notApplicable")]
    NotApplicable,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OptimizationProfile {
    Competitive,
    Balanced,
    RestoreDefaults,
}

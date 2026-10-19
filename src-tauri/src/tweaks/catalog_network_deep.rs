use super::metadata::{ImpactLevel, OptimizationProfile, RiskLevel, TweakCategory};
use super::model::TweakDefinition;

pub fn tweaks() -> Vec<TweakDefinition> {
    vec![TweakDefinition {
        id: "net_dns_cloudflare".into(),
        name: "Set DNS to Cloudflare (1.1.1.1)".into(),
        description: "Sets primary DNS on active Ethernet adapter.".into(),
        expected_benefit: "Faster DNS resolution for matchmaking endpoints.".into(),
        category: TweakCategory::Network,
        risk: RiskLevel::Medium,
        impact: ImpactLevel::Low,
        requires_reboot: false,
        profiles: vec![OptimizationProfile::Balanced],
        intel_only: false,
        amd_only: false,
        nvidia_only: false,
        amd_gpu_only: false,
        guidance_only: false,
        registry_writes: vec![],
        powershell: Some(
            r#"
$if = Get-NetAdapter -Physical | Where-Object Status -eq 'Up' | Select-Object -First 1
if (-not $if) { throw 'No active adapter' }
Set-DnsClientServerAddress -InterfaceIndex $if.ifIndex -ServerAddresses @('1.1.1.1','1.0.0.1')
"#
            .into(),
        ),
        revert_writes: vec![],
        detect: None,
    }]
}

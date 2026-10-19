use serde::{Deserialize, Serialize};

use crate::backup::{append_registry_backup, backup_registry_value, start_session};
use crate::error::{AppError, AppResult};
use crate::hardware::SystemReport;

use super::catalog::all_tweaks;
use super::detection::detect_tweak;
use super::metadata::{DetectedState, OptimizationProfile};
use super::model::{TweakCatalogEntry, TweakDefinition, TweakState};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyProfileResult {
    pub session_id: String,
    pub applied: Vec<String>,
    pub skipped: Vec<String>,
    pub guidance: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleTweakResult {
    pub enabled: bool,
    pub verified: bool,
    pub message: String,
}

static POWER_SNAP: std::sync::Mutex<Option<crate::platform::power_snapshot::PowerSnapshot>> =
    std::sync::Mutex::new(None);

pub fn hardware_matches(def: &TweakDefinition, report: &SystemReport) -> bool {
    let cpu = report.cpu.vendor.to_lowercase();
    let gpu = report.gpu.vendor.to_lowercase();
    if def.intel_only && !cpu.contains("intel") {
        return false;
    }
    if def.amd_only && !cpu.contains("amd") {
        return false;
    }
    if def.nvidia_only && !gpu.contains("nvidia") {
        return false;
    }
    if def.amd_gpu_only && !gpu.contains("amd") {
        return false;
    }
    true
}

pub fn find_tweak(id: &str) -> Option<TweakDefinition> {
    all_tweaks().into_iter().find(|t| t.id == id)
}

pub fn apply_tweak_by_id(id: &str, report: &SystemReport) -> AppResult<()> {
    let def = find_tweak(id).ok_or_else(|| AppError::msg(format!("Unknown tweak: {id}")))?;
    if def.guidance_only {
        return Err(AppError::msg("Guidance-only tweak cannot be applied"));
    }
    if !hardware_matches(&def, report) {
        return Err(AppError::msg("Tweak not applicable on this hardware"));
    }
    apply_special(&def, report)?;
    apply_registry_batch(&def, true)?;
    if let Some(ps) = &def.powershell {
        #[cfg(windows)]
        {
            let out = crate::platform::powershell::run(ps)?;
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                return Err(AppError::msg(format!("PowerShell apply failed: {}", stderr.trim())));
            }
        }
    }
    Ok(())
}

pub fn revert_single_tweak(id: &str) -> AppResult<()> {
    let def = find_tweak(id).ok_or_else(|| AppError::msg(format!("Unknown tweak: {id}")))?;
    revert_special(&def)?;
    let writes = if def.revert_writes.is_empty() {
        def.registry_writes.clone()
    } else {
        def.revert_writes.clone()
    };
    for w in writes {
        restore_registry_write(&w)?;
    }
    Ok(())
}

pub fn toggle_tweak(
    tweak_id: &str,
    enable: bool,
    report: &SystemReport,
) -> AppResult<ToggleTweakResult> {
    if enable {
        apply_tweak_by_id(tweak_id, report)?;
    } else {
        revert_single_tweak(tweak_id)?;
    }
    let def = find_tweak(tweak_id).unwrap();
    let detected = detect_tweak(&def);
    let verified = if enable {
        detected == DetectedState::On || detected == DetectedState::Unknown
    } else {
        detected == DetectedState::Off || detected == DetectedState::Unknown
    };
    Ok(ToggleTweakResult {
        enabled: enable,
        verified,
        message: format!("{} — detected {}", def.name, format_detected(detected)),
    })
}

pub fn apply_profile(profile: OptimizationProfile, report: &SystemReport) -> AppResult<ApplyProfileResult> {
    let tweaks = super::catalog::tweaks_for_profile(profile, report);
    let mut applied = Vec::new();
    let mut skipped = Vec::new();
    let mut guidance = Vec::new();
    let session = start_session("optimization profile", &[])?;
    for def in tweaks {
        if def.guidance_only {
            guidance.push(def.id.clone());
            continue;
        }
        match apply_tweak_by_id(&def.id, report) {
            Ok(()) => applied.push(def.id.clone()),
            Err(_) => skipped.push(def.id.clone()),
        }
    }
    Ok(ApplyProfileResult {
        session_id: session.id,
        applied,
        skipped,
        guidance,
    })
}

pub fn apply_custom_profile(profile_id: &str, report: &SystemReport) -> AppResult<ApplyProfileResult> {
    let store = super::custom_profiles::load_profiles()?;
    let profile = store
        .profiles
        .iter()
        .find(|p| p.id == profile_id)
        .ok_or_else(|| AppError::msg("Custom profile not found"))?;
    let session = start_session(&format!("custom:{}", profile.name), &profile.tweak_ids)?;
    let mut applied = Vec::new();
    let mut skipped = Vec::new();
    let guidance = Vec::new();
    for id in &profile.tweak_ids {
        match apply_tweak_by_id(id, report) {
            Ok(()) => applied.push(id.clone()),
            Err(_) => skipped.push(id.clone()),
        }
    }
    Ok(ApplyProfileResult {
        session_id: session.id,
        applied,
        skipped,
        guidance,
    })
}

pub fn get_catalog(report: &SystemReport) -> AppResult<Vec<TweakCatalogEntry>> {
    Ok(list_entries(report))
}

pub fn list_tweak_states(report: &SystemReport) -> AppResult<Vec<(TweakDefinition, TweakState)>> {
    Ok(list_entries(report)
        .into_iter()
        .map(|e| (e.definition, e.state))
        .collect())
}

fn list_entries(report: &SystemReport) -> Vec<TweakCatalogEntry> {
    all_tweaks()
        .into_iter()
        .filter(|d| hardware_matches(d, report))
        .map(|def| {
            let id = def.id.clone();
            let detected = detect_tweak(&def);
            let enabled = detected == DetectedState::On;
            TweakCatalogEntry {
                definition: def,
                state: TweakState {
                    id,
                    applied: enabled,
                    applied_at: None,
                    session_id: None,
                },
                detected,
                enabled,
            }
        })
        .collect()
}

fn format_detected(d: DetectedState) -> &'static str {
    match d {
        DetectedState::On => "on",
        DetectedState::Off => "off",
        DetectedState::Unknown => "unknown",
        DetectedState::NotApplicable => "n/a",
    }
}

fn apply_registry_batch(def: &TweakDefinition, backup: bool) -> AppResult<()> {
    for w in &def.registry_writes {
        if backup {
            let entry = backup_registry_value(&w.hive, &w.path, &w.value_name)?;
            if let Ok(manifest) = crate::backup::load_manifest() {
                if let Some(session) = manifest.sessions.last() {
                    let _ = append_registry_backup(&session.id, entry);
                }
            }
        }
        write_registry_write(w)?;
    }
    Ok(())
}

fn write_registry_write(w: &super::model::RegistryWrite) -> AppResult<()> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hive_key = match w.hive.to_uppercase().as_str() {
            "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
            "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
            _ => return Err(AppError::msg("Unsupported hive")),
        };
        let (key, _) = hive_key.create_subkey(&w.path)?;
        crate::backup::write_reg_value(&key, &w.value_name, &w.kind, Some(&w.data))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = w;
        Ok(())
    }
}

fn restore_registry_write(w: &super::model::RegistryWrite) -> AppResult<()> {
    write_registry_write(w)
}

fn apply_special(def: &TweakDefinition, report: &SystemReport) -> AppResult<()> {
    match def.id.as_str() {
        "power_ultimate" => {
            #[cfg(windows)]
            {
                if POWER_SNAP.lock().unwrap().is_none() {
                    *POWER_SNAP.lock().unwrap() = Some(crate::platform::power_snapshot::capture()?);
                }
                crate::platform::powercfg::set_ultimate_performance()?;
            }
        }
        "power_min_processor_100" => {
            #[cfg(windows)]
            {
                capture_power_snap_once()?;
                crate::platform::powercfg::set_processor_min_percent(100)?;
            }
        }
        "power_max_processor_100" => {
            #[cfg(windows)]
            {
                capture_power_snap_once()?;
                crate::platform::powercfg::set_processor_max_percent(100)?;
            }
        }
        "intel_core_parking_min" | "intel_core_parking" => {
            #[cfg(windows)]
            {
                capture_power_snap_once()?;
                crate::platform::powercfg::set_core_parking_min_percent(100)?;
            }
        }
        "amd_processor_boost" => {
            #[cfg(windows)]
            {
                capture_power_snap_once()?;
                crate::platform::powercfg::set_perfboost_mode(2)?;
            }
        }
        "net_tcp_autotun_normal" => crate::tweaks::netsh_ops::set_autotun_normal()?,
        "net_tcp_ecn_off" => crate::tweaks::netsh_ops::set_ecn_disabled()?,
        "net_tcp_rss_on" => crate::tweaks::netsh_ops::set_rss_enabled()?,
        "net_tcp_timestamps_off" => crate::tweaks::netsh_ops::set_timestamps_disabled()?,
        "net_initial_rto" => crate::tweaks::netsh_ops::set_initial_rto_ms(300)?,
        "defender_cs2_exclusion" => crate::tweaks::defender::add_cs2_folder_exclusion()?,
        "cs2_disable_fullscreen_opt" => {
            let w = crate::tweaks::cs2_compat::registry_write_for_fso()?;
            write_registry_write(&w)?;
        }
        "timer_resolution" => {
            crate::services::set_timer_policy(&crate::services::TimerPolicy {
                target_resolution_ms: 0.5,
                use_global_registry: true,
            })?;
            crate::services::start_timer_service(crate::services::TimerPolicy::default())?;
        }
        "cs2_preset" => apply_bundle(&["game_bar_off", "net_tcp_autotun_normal", "net_initial_rto"], report)?,
        "valorant_preset" => apply_bundle(&["game_bar_off", "net_tcp_ecn_off", "timer_resolution"], report)?,
        "fortnite_preset" => apply_bundle(&["power_min_processor_100", "game_bar_off"], report)?,
        _ => {}
    }
    Ok(())
}

fn apply_bundle(ids: &[&str], report: &SystemReport) -> AppResult<()> {
    for id in ids {
        let _ = apply_tweak_by_id(id, report);
    }
    Ok(())
}

fn capture_power_snap_once() -> AppResult<()> {
    #[cfg(windows)]
    {
        if POWER_SNAP.lock().unwrap().is_none() {
            *POWER_SNAP.lock().unwrap() = Some(crate::platform::power_snapshot::capture()?);
        }
    }
    Ok(())
}

fn revert_special(def: &TweakDefinition) -> AppResult<()> {
    match def.id.as_str() {
        "power_ultimate" | "power_min_processor_100" | "power_max_processor_100"
        | "intel_core_parking_min" | "intel_core_parking" | "amd_processor_boost" => {
            #[cfg(windows)]
            if let Some(snap) = POWER_SNAP.lock().unwrap().clone() {
                crate::platform::power_snapshot::restore(&snap)?;
            }
        }
        "net_initial_rto" => crate::tweaks::netsh_ops::revert_initial_rto()?,
        "defender_cs2_exclusion" => {
            #[cfg(windows)]
            {
                let _ = crate::platform::powershell::run(
                    r#"
$dirs = @()
try { $cs2 = (Get-ItemProperty 'HKCU:\Software\Valve\Steam' -ErrorAction SilentlyContinue).SteamPath } catch {}
# Best-effort: exclusions removed manually if needed
"#,
                );
            }
        }
        "cs2_disable_fullscreen_opt" => crate::tweaks::cs2_compat::revert_cs2_fso()?,
        "timer_resolution" => {
            crate::services::stop_timer_service()?;
            crate::services::set_timer_policy(&crate::services::TimerPolicy {
                target_resolution_ms: 15.0,
                use_global_registry: false,
            })?;
        }
        _ => {}
    }
    Ok(())
}

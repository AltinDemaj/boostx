use super::metadata::DetectedState;
use super::model::TweakDefinition;
use crate::error::AppResult;

#[cfg(windows)]
pub fn detect_tweak_state(def: &TweakDefinition) -> DetectedState {
    if let Some(state) = detect_by_id(def.id.as_str()) {
        return state;
    }
    if let Some(rule) = &def.detect {
        return detect_registry_rule(rule);
    }
    if let Some(w) = def.registry_writes.first() {
        let rule = super::model::TweakDetectRule {
            hive: w.hive.clone(),
            path: w.path.clone(),
            value_name: w.value_name.clone(),
            on_value: w.data.clone(),
        };
        return detect_registry_rule(&rule);
    }
    if def.guidance_only {
        return DetectedState::NotApplicable;
    }
    if def.powershell.is_some() {
        return DetectedState::Unknown;
    }
    DetectedState::Unknown
}

#[cfg(not(windows))]
pub fn detect_tweak_state(def: &TweakDefinition) -> DetectedState {
    if def.guidance_only {
        return DetectedState::NotApplicable;
    }
    DetectedState::Unknown
}

#[cfg(windows)]
fn power_min_state() -> DetectedState {
    match crate::platform::powercfg::read_processor_min_percent() {
        Ok(Some(n)) if n >= 100 => DetectedState::On,
        Ok(Some(_)) => DetectedState::Off,
        Ok(None) | Err(_) => DetectedState::Unknown,
    }
}

#[cfg(windows)]
fn power_max_state() -> DetectedState {
    match crate::platform::powercfg::read_processor_max_percent() {
        Ok(Some(n)) if n >= 100 => DetectedState::On,
        Ok(Some(_)) => DetectedState::Off,
        Ok(None) | Err(_) => DetectedState::Unknown,
    }
}

#[cfg(windows)]
fn amd_boost_state() -> DetectedState {
    match crate::platform::powercfg::read_perfboost_mode() {
        Ok(Some(n)) if n >= 2 => DetectedState::On,
        Ok(Some(_)) => DetectedState::Off,
        Ok(None) | Err(_) => DetectedState::Unknown,
    }
}

#[cfg(windows)]
fn detect_by_id(id: &str) -> Option<DetectedState> {
    use crate::platform::powercfg;

    match id {
        "power_ultimate" => Some(match powercfg::active_scheme_is_ultimate() {
            Ok(true) => DetectedState::On,
            Ok(false) => DetectedState::Off,
            Err(_) => DetectedState::Unknown,
        }),
        "power_min_processor_100" => Some(power_min_state()),
        "power_max_processor_100" => Some(power_max_state()),
        "intel_core_parking_min" | "intel_core_parking" => {
            Some(match powercfg::read_core_parking_min_percent() {
                Ok(Some(n)) if n >= 100 => DetectedState::On,
                Ok(Some(_)) => DetectedState::Off,
                Ok(None) | Err(_) => DetectedState::Unknown,
            })
        }
        "amd_processor_boost" => Some(amd_boost_state()),
        "net_tcp_autotun_normal" => Some(match crate::tweaks::netsh_ops::detect_autotun_normal() {
            Ok(true) => DetectedState::On,
            Ok(false) => DetectedState::Off,
            Err(_) => DetectedState::Unknown,
        }),
        "net_tcp_ecn_off" => Some(match crate::tweaks::netsh_ops::detect_ecn_disabled() {
            Ok(true) => DetectedState::On,
            Ok(false) => DetectedState::Off,
            Err(_) => DetectedState::Unknown,
        }),
        "net_tcp_rss_on" => Some(match crate::tweaks::netsh_ops::detect_rss_enabled() {
            Ok(true) => DetectedState::On,
            Ok(false) => DetectedState::Off,
            Err(_) => DetectedState::Unknown,
        }),
        "net_tcp_timestamps_off" => {
            Some(match crate::tweaks::netsh_ops::detect_timestamps_disabled() {
                Ok(true) => DetectedState::On,
                Ok(false) => DetectedState::Off,
                Err(_) => DetectedState::Unknown,
            })
        }
        "net_initial_rto" => Some(match crate::tweaks::netsh_ops::detect_initial_rto_ms() {
            Ok(Some(v)) if v <= 300 => DetectedState::On,
            Ok(Some(_)) => DetectedState::Off,
            Ok(None) => DetectedState::Off,
            Err(_) => DetectedState::Unknown,
        }),
        "defender_cs2_exclusion" => Some(match crate::tweaks::defender::cs2_install_dir_excluded() {
            Ok(true) => DetectedState::On,
            Ok(false) => DetectedState::Off,
            Err(_) => DetectedState::NotApplicable,
        }),
        "cs2_disable_fullscreen_opt" => Some(crate::tweaks::cs2_compat::detect_cs2_fso_disabled()),
        "game_bar_off" => Some(detect_game_bar_off()),
        "fullscreen_optimizations_off" => Some(detect_registry_rule(&super::model::TweakDetectRule {
            hive: "HKCU".into(),
            path: "System\\GameConfigStore".into(),
            value_name: "GameDVR_FSEBehaviorMode".into(),
            on_value: serde_json::json!(2),
        })),
        "nvidia_low_latency" => detect_nvidia_perf_level().ok(),
        "timer_resolution" => Some(match crate::services::measure_timer_resolution_us() {
            Ok(us) if us <= 1000.0 => DetectedState::On,
            Ok(_) => DetectedState::Off,
            Err(_) => DetectedState::Unknown,
        }),
        "hpet_bcdedit" => detect_bcdedit_platform_tick().ok(),
        "network_nagle_nic" => detect_nagle_on_primary_nic().ok(),
        "nic_power_saving_off" | "net_nic_advanced_competitive" => detect_nic_eee_disabled().ok(),
        "standby_list_clean" | "temp_cleanup" | "pagefile_system_managed" => Some(DetectedState::NotApplicable),
        "cs2_preset" | "valorant_preset" | "fortnite_preset" => Some(DetectedState::NotApplicable),
        _ => None,
    }
}

#[cfg(windows)]
fn detect_game_bar_off() -> DetectedState {
    let keys = [
        (
            "HKCU",
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\GameDVR",
            "GameBarEnabled",
            serde_json::json!(0),
        ),
        (
            "HKCU",
            "System\\GameConfigStore",
            "GameBarPresenceEnabled",
            serde_json::json!(0),
        ),
        (
            "HKCU",
            "SOFTWARE\\Microsoft\\GameBar",
            "AutoGameModeEnabled",
            serde_json::json!(0),
        ),
    ];
    let mut any = false;
    let mut all_on = true;
    for (hive, path, name, on) in keys {
        match read_registry_value(hive, path, name) {
            Ok(Some(v)) => {
                any = true;
                if !values_match(&v, &on) {
                    all_on = false;
                }
            }
            Ok(None) => all_on = false,
            Err(_) => return DetectedState::Unknown,
        }
    }
    if !any {
        return DetectedState::Off;
    }
    if all_on {
        DetectedState::On
    } else {
        DetectedState::Off
    }
}

#[cfg(windows)]
fn detect_bcdedit_platform_tick() -> AppResult<DetectedState> {
    let out = crate::platform::cmd::run_hidden_ok("bcdedit", &["/enum", "{current}"])?;
    let lower = out.to_lowercase();
    let on = lower.contains("useplatformtick") && lower.contains("yes");
    Ok(if on {
        DetectedState::On
    } else {
        DetectedState::Off
    })
}

#[cfg(windows)]
fn detect_nagle_on_primary_nic() -> AppResult<DetectedState> {
    let out = crate::platform::powershell::run(
        r#"
$k = Get-ChildItem 'HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces' | Select-Object -First 1
if (-not $k) { '0' } else {
  $v = (Get-ItemProperty $k.PSPath -ErrorAction SilentlyContinue).TcpAckFrequency
  if ($v -eq 1) { '1' } else { '0' }
}
"#,
    )?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let s = stdout.trim();
    Ok(if s == "1" {
        DetectedState::On
    } else {
        DetectedState::Off
    })
}

#[cfg(windows)]
fn detect_nic_eee_disabled() -> AppResult<DetectedState> {
    let out = crate::platform::powershell::run(
        r#"
$bad = Get-NetAdapter -Physical | ForEach-Object {
  $p = Get-NetAdapterAdvancedProperty -Name $_.Name -DisplayName 'Energy Efficient Ethernet' -ErrorAction SilentlyContinue
  if ($p -and $p.DisplayValue -match 'Enabled') { $true }
} | Where-Object { $_ }
if ($bad) { '0' } else { '1' }
"#,
    )?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let s = stdout.trim();
    Ok(if s == "1" {
        DetectedState::On
    } else {
        DetectedState::Off
    })
}

#[cfg(windows)]
fn detect_nvidia_perf_level() -> AppResult<DetectedState> {
    match read_registry_value(
        "HKLM",
        "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}\\0000",
        "PerfLevelSrc",
    )? {
        Some(v) if v == serde_json::json!(0x2222) || v == serde_json::json!(8738) => {
            Ok(DetectedState::On)
        }
        Some(_) => Ok(DetectedState::Off),
        None => Ok(DetectedState::Off),
    }
}

#[cfg(windows)]
fn detect_registry_rule(rule: &super::model::TweakDetectRule) -> DetectedState {
    match read_registry_value(&rule.hive, &rule.path, &rule.value_name) {
        Ok(Some(value)) => {
            if values_match(&value, &rule.on_value) {
                DetectedState::On
            } else {
                DetectedState::Off
            }
        }
        Ok(None) => DetectedState::Off,
        Err(_) => DetectedState::Unknown,
    }
}

#[cfg(windows)]
pub fn read_registry_value(
    hive: &str,
    path: &str,
    value_name: &str,
) -> AppResult<Option<serde_json::Value>> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hive_key = match hive.to_uppercase().as_str() {
        "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
        "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
        _ => return Ok(None),
    };
    let key = hive_key.open_subkey(path)?;
    if let Ok(v) = key.get_value::<u32, _>(value_name) {
        return Ok(Some(serde_json::json!(v)));
    }
    if let Ok(v) = key.get_value::<String, _>(value_name) {
        return Ok(Some(serde_json::json!(v)));
    }
    Ok(None)
}

fn values_match(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
        (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
            a.as_u64() == b.as_u64() || a.as_i64() == b.as_i64()
        }
        (serde_json::Value::String(a), serde_json::Value::String(b)) => {
            a.eq_ignore_ascii_case(b) || a == b
        }
        _ => actual == expected,
    }
}

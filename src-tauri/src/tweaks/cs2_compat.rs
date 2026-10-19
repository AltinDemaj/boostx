use crate::error::AppResult;
use crate::games::find_cs2_install;
use crate::tweaks::metadata::DetectedState;
use crate::tweaks::model::RegistryWrite;

const LAYER_VALUE: &str = "~ DISABLEDXMAXIMIZEDWINDOWEDMODE";
const LAYERS_PATH: &str = "Software\\Microsoft\\Windows NT\\CurrentVersion\\AppCompatFlags\\Layers";

pub fn cs2_exe_path() -> AppResult<std::path::PathBuf> {
    find_cs2_install()
}

pub fn registry_write_for_fso() -> AppResult<RegistryWrite> {
    let exe = cs2_exe_path()?;
    Ok(RegistryWrite {
        hive: "HKCU".into(),
        path: LAYERS_PATH.into(),
        value_name: exe.to_string_lossy().to_string(),
        kind: "REG_SZ".into(),
        data: serde_json::json!(LAYER_VALUE),
    })
}

pub fn detect_cs2_fso_disabled() -> DetectedState {
    #[cfg(not(windows))]
    return DetectedState::NotApplicable;

    #[cfg(windows)]
    {
        match cs2_exe_path() {
            Ok(exe) => {
                let name = exe.to_string_lossy().to_string();
                match crate::tweaks::detectors::read_registry_value("HKCU", LAYERS_PATH, &name) {
                    Ok(Some(v)) => {
                        let s = v.as_str().unwrap_or("");
                        if s.contains("DISABLEDXMAXIMIZEDWINDOWEDMODE") {
                            DetectedState::On
                        } else {
                            DetectedState::Off
                        }
                    }
                    Ok(None) => DetectedState::Off,
                    Err(_) => DetectedState::Unknown,
                }
            }
            Err(_) => DetectedState::NotApplicable,
        }
    }
}

pub fn revert_cs2_fso() -> AppResult<()> {
    let exe = cs2_exe_path()?;
    let name = exe.to_string_lossy().to_string();
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey_with_flags(LAYERS_PATH, KEY_SET_VALUE) {
            let _ = key.delete_value(&name);
        }
    }
    #[cfg(not(windows))]
    let _ = name;
    Ok(())
}

use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryBackupEntry {
    pub hive: String,
    pub path: String,
    pub value_name: String,
    pub kind: String,
    pub data: Option<serde_json::Value>,
    pub existed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSession {
    pub id: String,
    pub created_at: String,
    pub restore_point_id: Option<String>,
    pub registry_entries: Vec<RegistryBackupEntry>,
    pub tweak_ids: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupManifest {
    pub sessions: Vec<BackupSession>,
}

pub fn data_dir() -> AppResult<PathBuf> {
    let base = dirs::data_local_dir()
        .ok_or_else(|| AppError::msg("Could not resolve local data directory"))?;
    let path = base.join("BoostX");
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn manifest_path() -> AppResult<PathBuf> {
    Ok(data_dir()?.join("backups.json"))
}

pub fn load_manifest() -> AppResult<BackupManifest> {
    let path = manifest_path()?;
    if !path.exists() {
        return Ok(BackupManifest { sessions: vec![] });
    }
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn save_manifest(manifest: &BackupManifest) -> AppResult<()> {
    let path = manifest_path()?;
    let raw = serde_json::to_string_pretty(manifest)?;
    fs::write(path, raw)?;
    Ok(())
}

pub fn create_restore_point(description: &str) -> AppResult<Option<String>> {
    #[cfg(windows)]
    {
        use std::process::Command;

        let ps = format!(
            "Checkpoint-Computer -Description '{}' -RestorePointType 'MODIFY_SETTINGS'",
            description.replace('\'', "''")
        );
        let output = crate::platform::powershell::run(&ps)?;

        if output.status.success() {
            return Ok(Some(Uuid::new_v4().to_string()));
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("disabled") || stderr.contains("Frequency") {
            return Ok(None);
        }
        return Ok(None);
    }
    #[cfg(not(windows))]
    {
        let _ = description;
        Ok(Some("demo-restore-point".into()))
    }
}

pub fn backup_registry_value(
    hive: &str,
    path: &str,
    value_name: &str,
) -> AppResult<RegistryBackupEntry> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hive_key = match hive.to_uppercase().as_str() {
            "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
            "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKCR" => RegKey::predef(HKEY_CLASSES_ROOT),
            _ => return Err(AppError::msg(format!("Unsupported hive: {hive}"))),
        };

        let key = hive_key.open_subkey(path);
        match key {
            Ok(k) => {
                let existed = true;
                let data = read_reg_value(&k, value_name)?;
                Ok(RegistryBackupEntry {
                    hive: hive.to_string(),
                    path: path.to_string(),
                    value_name: value_name.to_string(),
                    kind: data.0,
                    data: data.1,
                    existed,
                })
            }
            Err(_) => Ok(RegistryBackupEntry {
                hive: hive.to_string(),
                path: path.to_string(),
                value_name: value_name.to_string(),
                kind: "missing".into(),
                data: None,
                existed: false,
            }),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (hive, path, value_name);
        Ok(RegistryBackupEntry {
            hive: hive.to_string(),
            path: path.to_string(),
            value_name: value_name.to_string(),
            kind: "demo".into(),
            data: None,
            existed: false,
        })
    }
}

#[cfg(windows)]
fn read_reg_value(
    key: &winreg::RegKey,
    value_name: &str,
) -> AppResult<(String, Option<serde_json::Value>)> {
    use winreg::RegValue;

    match key.get_raw_value(value_name) {
        Ok(RegValue { vtype, bytes }) => {
            let kind = format!("REG_{:?}", vtype);
            let data = decode_reg_bytes(vtype, &bytes);
            Ok((kind, data))
        }
        Err(_) => Ok(("missing".into(), None)),
    }
}

#[cfg(windows)]
fn decode_reg_bytes(vtype: winreg::enums::RegType, bytes: &[u8]) -> Option<serde_json::Value> {
    use winreg::enums::*;

    match vtype {
        REG_DWORD => {
            if bytes.len() >= 4 {
                let v = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                Some(serde_json::json!(v))
            } else {
                None
            }
        }
        REG_QWORD => {
            if bytes.len() >= 8 {
                let v = u64::from_le_bytes([
                    bytes[0],
                    bytes[1],
                    bytes[2],
                    bytes[3],
                    bytes[4],
                    bytes[5],
                    bytes[6],
                    bytes[7],
                ]);
                Some(serde_json::json!(v))
            } else {
                None
            }
        }
        REG_SZ | REG_EXPAND_SZ => {
            let s = String::from_utf16_lossy(
                &bytes
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect::<Vec<_>>(),
            );
            Some(serde_json::Value::String(s.trim_end_matches('\0').to_string()))
        }
        _ => Some(serde_json::json!(bytes)),
    }
}

pub fn restore_registry_entry(entry: &RegistryBackupEntry) -> AppResult<()> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hive_key = match entry.hive.to_uppercase().as_str() {
            "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
            "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
            _ => return Err(AppError::msg("Unsupported hive for restore")),
        };

        if !entry.existed {
            let key = hive_key.open_subkey_with_flags(&entry.path, KEY_SET_VALUE)?;
            let _ = key.delete_value(&entry.value_name);
            return Ok(());
        }

        let (key, _) = hive_key.create_subkey(&entry.path)?;
        write_reg_value(&key, &entry.value_name, &entry.kind, entry.data.as_ref())?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = entry;
        Ok(())
    }
}

#[cfg(windows)]
pub(crate) fn write_reg_value(
    key: &winreg::RegKey,
    value_name: &str,
    kind: &str,
    data: Option<&serde_json::Value>,
) -> AppResult<()> {
    use winreg::enums::*;

    let kind_upper = kind.to_uppercase();
    match kind_upper.as_str() {
        k if k.contains("DWORD") => {
            let v = data.and_then(|d| d.as_u64()).unwrap_or(0) as u32;
            key.set_value(value_name, &v)?;
        }
        k if k.contains("QWORD") => {
            let v = data.and_then(|d| d.as_u64()).unwrap_or(0);
            key.set_value(value_name, &v)?;
        }
        k if k.contains("SZ") || k == "STRING" => {
            let s = data
                .and_then(|d| d.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            key.set_value(value_name, &s)?;
        }
        _ => {}
    }
    Ok(())
}

pub fn start_session(note: &str, tweak_ids: &[String]) -> AppResult<BackupSession> {
    let restore_id = create_restore_point("BoostX optimization session")?;
    let session = BackupSession {
        id: Uuid::new_v4().to_string(),
        created_at: Utc::now().to_rfc3339(),
        restore_point_id: restore_id,
        registry_entries: vec![],
        tweak_ids: tweak_ids.to_vec(),
        note: note.to_string(),
    };
    let mut manifest = load_manifest()?;
    manifest.sessions.push(session.clone());
    save_manifest(&manifest)?;
    Ok(session)
}

pub fn append_registry_backup(session_id: &str, entry: RegistryBackupEntry) -> AppResult<()> {
    let mut manifest = load_manifest()?;
    let session = manifest
        .sessions
        .iter_mut()
        .find(|s| s.id == session_id)
        .ok_or_else(|| AppError::msg("Backup session not found"))?;
    session.registry_entries.push(entry);
    save_manifest(&manifest)?;
    Ok(())
}

pub fn revert_session(session_id: &str) -> AppResult<()> {
    let manifest = load_manifest()?;
    let session = manifest
        .sessions
        .iter()
        .find(|s| s.id == session_id)
        .ok_or_else(|| AppError::msg("Backup session not found"))?;

    for entry in session.registry_entries.iter().rev() {
        restore_registry_entry(entry)?;
    }
    Ok(())
}

pub fn revert_all() -> AppResult<u32> {
    let manifest = load_manifest()?;
    let count = manifest.sessions.len() as u32;
    for session in manifest.sessions.iter().rev() {
        for entry in session.registry_entries.iter().rev() {
            restore_registry_entry(entry)?;
        }
    }
    save_manifest(&BackupManifest { sessions: vec![] })?;
    Ok(count)
}

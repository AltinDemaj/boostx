use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomProfile {
    pub id: String,
    pub name: String,
    pub tweak_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CustomProfileStore {
    pub profiles: Vec<CustomProfile>,
}

fn store_path() -> AppResult<PathBuf> {
    Ok(crate::backup::data_dir()?.join("custom_profiles.json"))
}

pub fn load_profiles() -> AppResult<CustomProfileStore> {
    let path = store_path()?;
    if !path.exists() {
        return Ok(CustomProfileStore::default());
    }
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn save_profiles(store: &CustomProfileStore) -> AppResult<()> {
    let path = store_path()?;
    fs::write(path, serde_json::to_string_pretty(store)?)?;
    Ok(())
}

pub fn upsert_profile(name: String, tweak_ids: Vec<String>) -> AppResult<CustomProfile> {
    let mut store = load_profiles()?;
    let now = chrono::Utc::now().to_rfc3339();
    if let Some(existing) = store.profiles.iter_mut().find(|p| p.name == name) {
        existing.tweak_ids = tweak_ids;
        existing.updated_at = now.clone();
        return Ok(existing.clone());
    }
    let profile = CustomProfile {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        tweak_ids,
        created_at: now.clone(),
        updated_at: now,
    };
    store.profiles.push(profile.clone());
    save_profiles(&store)?;
    Ok(profile)
}

pub fn rename_profile(id: &str, new_name: String) -> AppResult<CustomProfile> {
    let mut store = load_profiles()?;
    if store.profiles.iter().any(|p| p.name == new_name && p.id != id) {
        return Err(AppError::msg("A profile with that name already exists"));
    }
    let profile = store
        .profiles
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::msg("Profile not found"))?;
    profile.name = new_name;
    profile.updated_at = chrono::Utc::now().to_rfc3339();
    let cloned = profile.clone();
    save_profiles(&store)?;
    Ok(cloned)
}

pub fn delete_profile(id: &str) -> AppResult<()> {
    let mut store = load_profiles()?;
    let len = store.profiles.len();
    store.profiles.retain(|p| p.id != id);
    if store.profiles.len() == len {
        return Err(AppError::msg("Profile not found"));
    }
    save_profiles(&store)?;
    Ok(())
}

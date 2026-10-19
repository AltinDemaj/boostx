use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledGame {
    pub app_id: String,
    pub name: String,
    pub install_dir: String,
    pub platform: String,
}

#[cfg(windows)]
pub fn find_cs2_install() -> AppResult<std::path::PathBuf> {
    for library in steam_library_roots()? {
        let common = library.join("steamapps").join("common");
        for entry in std::fs::read_dir(&common)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if name.contains("counter-strike") {
                let exe = path.join("game").join("bin").join("win64").join("cs2.exe");
                if exe.exists() {
                    return Ok(exe);
                }
                let legacy = path.join("cs2.exe");
                if legacy.exists() {
                    return Ok(legacy);
                }
            }
        }
    }
    Err(AppError::msg("CS2 not installed"))
}

#[cfg(not(windows))]
pub fn find_cs2_install() -> AppResult<std::path::PathBuf> {
    Err(AppError::msg("CS2 not installed"))
}

pub fn detect_installed_games() -> AppResult<Vec<InstalledGame>> {
    #[cfg(windows)]
    {
        let mut games = Vec::new();
        for library in steam_library_roots()? {
            games.extend(scan_library(&library)?);
        }
        if games.is_empty() {
            games.extend(demo_games());
        }
        Ok(dedupe_games(games))
    }
    #[cfg(not(windows))]
    {
        Ok(demo_games())
    }
}

#[cfg(windows)]
fn steam_library_roots() -> AppResult<Vec<PathBuf>> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let steam_key = hkcu.open_subkey("Software\\Valve\\Steam");
    let steam_path = match steam_key {
        Ok(k) => k
            .get_value::<String, _>("SteamPath")
            .or_else(|_| k.get_value::<String, _>("InstallPath"))
            .map_err(|e| AppError::msg(e.to_string()))?,
        Err(_) => return Ok(vec![]),
    };

    let mut roots = vec![PathBuf::from(&steam_path)];
    let vdf = Path::new(&steam_path).join("steamapps").join("libraryfolders.vdf");
    if vdf.exists() {
        let text = fs::read_to_string(vdf)?;
        let re = Regex::new(r#""path"\s+"([^"]+)""#).unwrap();
        for cap in re.captures_iter(&text) {
            let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let normalized = path.replace("\\\\", "\\");
            roots.push(PathBuf::from(normalized));
        }
    }
    Ok(roots)
}

#[cfg(windows)]
fn scan_library(root: &Path) -> AppResult<Vec<InstalledGame>> {
    let apps = root.join("steamapps");
    let common = apps.join("common");
    if !common.exists() {
        return Ok(vec![]);
    }

    let mut games = Vec::new();
    for entry in fs::read_dir(&common)? {
        let path = entry?.path();
        if !path.is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let app_id = guess_app_id(&name);
        games.push(InstalledGame {
            app_id,
            name: friendly_name(&name),
            install_dir: path.to_string_lossy().to_string(),
            platform: "Steam".into(),
        });
    }
    Ok(games)
}

fn guess_app_id(folder_name: &str) -> String {
    let lower = folder_name.to_lowercase();
    if lower.contains("counter-strike") || lower == "csgo" {
        "730".into()
    } else if lower.contains("valorant") {
        "valorant".into()
    } else if lower.contains("fortnite") {
        "fortnite".into()
    } else {
        "unknown".into()
    }
}

fn friendly_name(folder_name: &str) -> String {
    if folder_name.to_lowercase().contains("counter-strike") {
        "Counter-Strike 2".into()
    } else {
        folder_name.to_string()
    }
}

fn demo_games() -> Vec<InstalledGame> {
    vec![
        InstalledGame {
            app_id: "730".into(),
            name: "Counter-Strike 2 (demo)".into(),
            install_dir: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Counter-Strike Global Offensive".into(),
            platform: "Steam".into(),
        },
        InstalledGame {
            app_id: "valorant".into(),
            name: "VALORANT (not auto-detected on Linux build)".into(),
            install_dir: "".into(),
            platform: "Riot".into(),
        },
    ]
}

fn dedupe_games(games: Vec<InstalledGame>) -> Vec<InstalledGame> {
    let mut seen = std::collections::HashSet::new();
    games
        .into_iter()
        .filter(|g| seen.insert(g.name.clone()))
        .collect()
}

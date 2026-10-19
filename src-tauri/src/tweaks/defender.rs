use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};
use crate::platform::cmd::run_hidden_ok;

#[cfg(windows)]
pub fn cs2_install_path_exists() -> bool {
    crate::games::find_cs2_install().is_ok()
}

#[cfg(not(windows))]
pub fn cs2_install_path_exists() -> bool {
    false
}

#[cfg(windows)]
fn cs2_exclusion_dirs() -> AppResult<Vec<PathBuf>> {
    let cs2 = crate::games::find_cs2_install()?;
    let mut dirs = Vec::new();
    if let Some(win64) = cs2.parent() {
        if let Some(bin) = win64.parent() {
            if let Some(game) = bin.parent() {
                dirs.push(game.to_path_buf());
                if let Some(root) = game.parent() {
                    dirs.push(root.to_path_buf());
                }
            }
        }
    }
    if dirs.is_empty() {
        return Err(AppError::msg("Invalid CS2 path"));
    }
    Ok(dirs)
}

#[cfg(windows)]
pub fn cs2_install_dir_excluded() -> AppResult<bool> {
    for dir in cs2_exclusion_dirs()? {
        if path_is_defender_excluded(&dir)? {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(not(windows))]
pub fn cs2_install_dir_excluded() -> AppResult<bool> {
    Ok(false)
}

#[cfg(windows)]
pub fn add_cs2_folder_exclusion() -> AppResult<()> {
    let dirs = cs2_exclusion_dirs()?;
    for dir in &dirs {
        let path = dir.to_string_lossy().replace('\'', "''");
        let script = format!("Add-MpPreference -ExclusionPath '{path}' -ErrorAction Stop");
        let output = crate::platform::powershell::run(&script)?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::msg(format!("Add-MpPreference failed: {}", stderr.trim())));
        }
    }
    if !cs2_install_dir_excluded()? {
        return Err(AppError::msg("Defender exclusion was not applied"));
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn add_cs2_folder_exclusion() -> AppResult<()> {
    Ok(())
}

#[cfg(windows)]
fn path_is_defender_excluded(path: &Path) -> AppResult<bool> {
    let norm = path.to_string_lossy().replace('\'', "''");
    let out = run_hidden_ok(
        "powershell",
        &[
            "-NoProfile",
            "-Command",
            &format!(
                r#"
$p = (Resolve-Path '{norm}' -ErrorAction SilentlyContinue).Path
if (-not $p) {{ exit 2 }}
$p = $p.TrimEnd('\').ToLower()
$eps = @(Get-MpPreference -ErrorAction SilentlyContinue).ExclusionPath
if (-not $eps) {{ '0'; exit }}
foreach ($e in $eps) {{
  $x = $e.TrimEnd('\').ToLower()
  if ($x -eq $p -or $x.StartsWith($p) -or $p.StartsWith($x)) {{ '1'; exit }}
}}
'0'
"#
            ),
        ],
    )?;
    Ok(out.trim() == "1")
}

#[cfg(not(windows))]
fn path_is_defender_excluded(_path: &Path) -> AppResult<bool> {
    Ok(false)
}

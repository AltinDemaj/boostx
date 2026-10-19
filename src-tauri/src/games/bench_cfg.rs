use std::fs;
use std::path::PathBuf;

use crate::error::{AppError, AppResult};
use crate::games::find_cs2_install;

const BENCH_CFG: &str = include_str!("../../resources/boostx_bench.cfg");

/// Installs `boostx_bench.cfg` into the CS2 cfg folder and returns its path.
pub fn install_boostx_bench_cfg() -> AppResult<PathBuf> {
    let cs2_exe = find_cs2_install()?;
    let cfg_dir = cs2_cfg_dir(&cs2_exe)?;
    fs::create_dir_all(&cfg_dir)?;
    let dest = cfg_dir.join("boostx_bench.cfg");
    fs::write(&dest, BENCH_CFG)?;
    Ok(dest)
}

fn cs2_cfg_dir(cs2_exe: &std::path::Path) -> AppResult<PathBuf> {
    // .../Counter-Strike Global Offensive/game/bin/win64/cs2.exe -> .../game/csgo/cfg
    let game = cs2_exe
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or_else(|| AppError::msg("Unexpected CS2 install layout"))?;
    let cfg = game.join("csgo").join("cfg");
    if cfg.exists() {
        return Ok(cfg);
    }
    Err(AppError::msg("CS2 cfg folder not found"))
}

use std::fs;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeModeSession {
    pub id: String,
    pub created_at: String,
    pub baseline_score: u32,
    pub baseline_avg_fps: Option<f32>,
    pub baseline_p1_fps: Option<f32>,
    pub tweak_ids: Vec<String>,
    pub confirmed: bool,
}

fn path() -> AppResult<std::path::PathBuf> {
    Ok(crate::backup::data_dir()?.join("safe_mode.json"))
}

pub fn load() -> AppResult<Option<SafeModeSession>> {
    let p = path()?;
    if !p.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&fs::read_to_string(p)?)?))
}

fn save(session: &SafeModeSession) -> AppResult<()> {
    fs::write(path()?, serde_json::to_string_pretty(session)?)?;
    Ok(())
}

pub fn arm_after_tweaks(tweak_ids: Vec<String>, baseline_score: u32) -> AppResult<()> {
    let session = SafeModeSession {
        id: Uuid::new_v4().to_string(),
        created_at: Utc::now().to_rfc3339(),
        baseline_score,
        baseline_avg_fps: None,
        baseline_p1_fps: None,
        tweak_ids,
        confirmed: false,
    };
    save(&session)
}

pub fn confirm_session() -> AppResult<()> {
    if let Some(mut s) = load()? {
        s.confirmed = true;
        save(&s)?;
    }
    Ok(())
}

pub fn evaluate_post_score(score: u32) -> AppResult<Option<String>> {
    let Some(session) = load()? else {
        return Ok(None);
    };
    if session.confirmed {
        return Ok(None);
    }
    if score + 3 < session.baseline_score {
        return Ok(Some(format!(
            "Performance score dropped {} → {} — consider reverting recent tweaks.",
            session.baseline_score,
            score
        )));
    }
    Ok(None)
}

pub fn evaluate_post_benchmark(avg_fps: f32, p1_fps: f32) -> AppResult<Option<String>> {
    let Some(session) = load()? else {
        return Ok(None);
    };
    if session.confirmed {
        return Ok(None);
    }
    if let (Some(base_avg), Some(base_p1)) = (session.baseline_avg_fps, session.baseline_p1_fps) {
        if avg_fps + 2.0 < base_avg || p1_fps + 2.0 < base_p1 {
            return Ok(Some(format!(
                "CS2 benchmark regression (avg {base_avg:.0}→{avg_fps:.0}, 1% {base_p1:.0}→{p1_fps:.0}) — revert recommended."
            )));
        }
    } else {
        let mut s = session;
        s.baseline_avg_fps = Some(avg_fps);
        s.baseline_p1_fps = Some(p1_fps);
        save(&s)?;
    }
    Ok(None)
}

pub fn clear() -> AppResult<()> {
    let p = path()?;
    if p.exists() {
        fs::remove_file(p)?;
    }
    Ok(())
}

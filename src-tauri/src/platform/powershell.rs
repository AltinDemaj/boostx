use std::process::Output;

use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::platform::cmd::run_hidden;

pub fn run(script: &str) -> AppResult<Output> {
    run_hidden(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script],
    )
}

pub fn run_json(script: &str) -> AppResult<Value> {
    let wrapped = format!("{script} | ConvertTo-Json -Compress -Depth 8");
    let out = run(&wrapped)?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(AppError::msg(format!("PowerShell failed: {}", stderr.trim())));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(trimmed)
        .map_err(|e| AppError::msg(format!("PowerShell JSON parse: {e} ({trimmed})")))
}

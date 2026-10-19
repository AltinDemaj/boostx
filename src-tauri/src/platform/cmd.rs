use std::process::{Command, Output};

use crate::error::{AppError, AppResult};

#[cfg(windows)]
pub fn run_hidden(program: &str, args: &[&str]) -> AppResult<Output> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    Command::new(program)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| AppError::msg(format!("Failed to run {program}: {e}")))
}

#[cfg(not(windows))]
pub fn run_hidden(program: &str, args: &[&str]) -> AppResult<Output> {
    Command::new(program)
        .args(args)
        .output()
        .map_err(|e| AppError::msg(format!("Failed to run {program}: {e}")))
}

pub fn run_hidden_ok(program: &str, args: &[&str]) -> AppResult<String> {
    let out = run_hidden(program, args)?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    if !out.status.success() {
        return Err(AppError::msg(format!(
            "{program} failed ({}): {} {}",
            out.status,
            stderr.trim(),
            stdout.trim()
        )));
    }
    Ok(stdout)
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[cfg(windows)]
use crate::platform::powershell::run;

#[cfg(not(windows))]
fn run(_script: &str) -> Result<std::process::Output, std::io::Error> {
    std::process::Command::new("sh")
        .args(["-c", "printf 15000"])
        .output()
}

static TIMER_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerPolicy {
    pub target_resolution_ms: f64,
    pub use_global_registry: bool,
}

impl Default for TimerPolicy {
    fn default() -> Self {
        Self {
            target_resolution_ms: 0.5,
            use_global_registry: true,
        }
    }
}

pub fn set_timer_policy(policy: &TimerPolicy) -> AppResult<()> {
    let p = crate::backup::data_dir()?.join("timer_policy.json");
    std::fs::write(p, serde_json::to_string_pretty(policy)?)?;
    if policy.use_global_registry {
        #[cfg(windows)]
        {
            let _ = run(
                r#"
New-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel' -Name GlobalTimerResolutionRequests -Value 1 -PropertyType DWord -Force | Out-Null
"#,
            );
        }
    }
    Ok(())
}

pub fn measure_timer_resolution_us() -> AppResult<f64> {
    #[cfg(windows)]
    {
        let out = run(
            r#"
Add-Type @"
using System.Runtime.InteropServices;
public class BxTimer {
  [DllImport("ntdll.dll")] public static extern int NtQueryTimerResolution(out uint Min, out uint Max, out uint Cur);
}
"@
[uint32]$min=0; [uint32]$max=0; [uint32]$cur=0
[BxTimer]::NtQueryTimerResolution([ref]$min,[ref]$max,[ref]$cur) | Out-Null
$cur
"#,
        )?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        let s = stdout.trim();
        let hundred_ns = s.parse::<f64>().unwrap_or(15_000.0);
        Ok(hundred_ns / 10.0)
    }
    #[cfg(not(windows))]
    {
        Ok(15_000.0)
    }
}

pub fn start_timer_service(policy: TimerPolicy) -> AppResult<()> {
    if TIMER_ACTIVE.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    let desired = (policy.target_resolution_ms * 10_000.0) as u32;
    thread::spawn(move || {
        while TIMER_ACTIVE.load(Ordering::SeqCst) {
            #[cfg(windows)]
            {
                let _ = run(&format!(
                    r#"
Add-Type @"
using System.Runtime.InteropServices;
public class BxTimerSet {{
  [DllImport("ntdll.dll")] public static extern int NtSetTimerResolution(uint Desired, bool Set, out uint Current);
}}
"@
[uint32]$c=0
[BxTimerSet]::NtSetTimerResolution({desired}, $true, [ref]$c) | Out-Null
"#
                ));
            }
            thread::sleep(Duration::from_secs(2));
        }
    });
    Ok(())
}

pub fn stop_timer_service() -> AppResult<()> {
    TIMER_ACTIVE.store(false, Ordering::SeqCst);
    Ok(())
}

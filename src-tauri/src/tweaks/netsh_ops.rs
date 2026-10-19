use crate::error::{AppError, AppResult};
use crate::platform::cmd::run_hidden_ok;

fn netsh_tcp_show() -> AppResult<String> {
    run_hidden_ok("netsh", &["interface", "tcp", "show", "global"])
}

pub fn detect_autotun_normal() -> AppResult<bool> {
    let out = netsh_tcp_show()?.to_lowercase();
    Ok(out.contains("receive window auto-tuning level") && out.contains("normal"))
}

pub fn detect_ecn_disabled() -> AppResult<bool> {
    let out = netsh_tcp_show()?.to_lowercase();
    Ok(out.contains("ecncapability") && out.contains("disabled"))
}

pub fn detect_rss_enabled() -> AppResult<bool> {
    let out = netsh_tcp_show()?.to_lowercase();
    Ok(out.contains("receive-side scaling state") && out.contains("enabled"))
}

pub fn detect_timestamps_disabled() -> AppResult<bool> {
    let out = netsh_tcp_show()?.to_lowercase();
    Ok(out.contains("timestamps") && out.contains("disabled"))
}

pub fn set_autotun_normal() -> AppResult<()> {
    run_hidden_ok("netsh", &["interface", "tcp", "set", "global", "autotuninglevel=normal"])?;
    Ok(())
}

pub fn set_ecn_disabled() -> AppResult<()> {
    run_hidden_ok("netsh", &["interface", "tcp", "set", "global", "ecncapability=disabled"])?;
    Ok(())
}

pub fn set_rss_enabled() -> AppResult<()> {
    run_hidden_ok("netsh", &["interface", "tcp", "set", "global", "rss=enabled"])?;
    Ok(())
}

pub fn set_timestamps_disabled() -> AppResult<()> {
    run_hidden_ok("netsh", &["interface", "tcp", "set", "global", "timestamps=disabled"])?;
    Ok(())
}

pub fn detect_initial_rto_ms() -> AppResult<Option<u32>> {
    #[cfg(windows)]
    {
        let out = crate::platform::powershell::run(
            r#"
$k = Get-ChildItem 'HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces' -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $k) { exit 1 }
$v = (Get-ItemProperty $k.PSPath -ErrorAction SilentlyContinue).TcpInitialRto
if ($null -eq $v) { '0' } else { [string]$v }
"#,
        )?;
        if !out.status.success() {
            return Ok(None);
        }
        let s = String::from_utf8_lossy(&out.stdout).trim();
        if s == "0" || s.is_empty() {
            return Ok(None);
        }
        return Ok(s.parse().ok());
    }
    #[cfg(not(windows))]
    {
        Ok(None)
    }
}

pub fn set_initial_rto_ms(ms: u32) -> AppResult<()> {
    #[cfg(windows)]
    {
        let script = format!(
            r#"
$ifaces = Get-ChildItem 'HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces'
if (-not $ifaces) {{ throw 'No interfaces' }}
foreach ($k in $ifaces) {{
  New-ItemProperty -Path $k.PSPath -Name 'TcpInitialRto' -Value {ms} -PropertyType DWord -Force | Out-Null
}}
"#,
            ms = ms
        );
        let out = crate::platform::powershell::run(&script)?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(AppError::msg(format!("TcpInitialRto failed: {}", stderr.trim())));
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = ms;
        Ok(())
    }
}

pub fn revert_initial_rto() -> AppResult<()> {
    #[cfg(windows)]
    {
        let script = r#"
$ifaces = Get-ChildItem 'HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces' -ErrorAction SilentlyContinue
foreach ($k in $ifaces) {
  Remove-ItemProperty -Path $k.PSPath -Name 'TcpInitialRto' -ErrorAction SilentlyContinue
}
"#;
        let _ = crate::platform::powershell::run(script)?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Ok(())
    }
}

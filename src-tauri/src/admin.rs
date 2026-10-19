use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElevationStatus {
    pub is_windows: bool,
    pub is_elevated: bool,
    pub message: String,
}

const ELEVATION_RETRY_ENV: &str = "BOOSTX_ELEVATED_RETRY";

pub fn elevation_status() -> ElevationStatus {
    #[cfg(windows)]
    {
        let elevated = is_process_elevated();
        ElevationStatus {
            is_windows: true,
            is_elevated: elevated,
            message: if elevated {
                "Running with administrator privileges.".into()
            } else {
                "Restart BoostX as administrator to apply system-wide tweaks.".into()
            },
        }
    }
    #[cfg(not(windows))]
    {
        ElevationStatus {
            is_windows: false,
            is_elevated: false,
            message: "BoostX targets Windows. This build shows demo data on non-Windows hosts.".into(),
        }
    }
}

#[cfg(windows)]
fn is_process_elevated() -> bool {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        );
        ok.is_ok() && elevation.TokenIsElevated != 0
    }
}

/// Re-launch the current executable with `runas` (UAC prompt). Returns true if a new process was started.
#[cfg(windows)]
pub fn request_elevation_restart() -> Result<bool, String> {
    use std::env;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    if is_process_elevated() {
        return Ok(false);
    }
    let exe = env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    env::set_var(ELEVATION_RETRY_ENV, "1");

    let op = wide("runas");
    let file = wide(&exe.to_string_lossy());
    let params = wide("");
    let dir_w = wide(&dir);

    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(op.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR(params.as_ptr()),
            PCWSTR(dir_w.as_ptr()),
            SW_SHOW,
        )
    };
    if result.0 as isize <= 32 {
        return Err(format!("ShellExecute runas failed (code {})", result.0 as isize));
    }
    Ok(true)
}

#[cfg(not(windows))]
pub fn request_elevation_restart() -> Result<bool, String> {
    Ok(false)
}

pub fn should_prompt_elevation_on_start() -> bool {
    #[cfg(windows)]
    {
        !is_process_elevated() && std::env::var(ELEVATION_RETRY_ENV).is_err()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

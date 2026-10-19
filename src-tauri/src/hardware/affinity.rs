use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[cfg(windows)]
use crate::platform::powershell::run_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreTopology {
    pub performance_cores: Vec<u32>,
    pub efficient_cores: Vec<u32>,
    pub note: String,
}

impl Default for CoreTopology {
    fn default() -> Self {
        Self {
            performance_cores: vec![0, 1],
            efficient_cores: vec![],
            note: "Default fallback topology".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameWatcherConfig {
    pub enabled: bool,
    pub process_names: Vec<String>,
    pub use_performance_cores_only: bool,
    pub priority_class: String,
}

impl Default for GameWatcherConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            process_names: vec!["cs2.exe".into()],
            use_performance_cores_only: true,
            priority_class: "high".into(),
        }
    }
}

pub fn detect_core_topology() -> AppResult<CoreTopology> {
    #[cfg(windows)]
    {
        let script = r#"
$phys = (Get-CimInstance Win32_Processor | Measure-Object -Property NumberOfCores -Sum).Sum
$log = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
$perf = @()
$eff = @()
try {
  $sets = Get-CimInstance -Namespace root/cimv2 -ClassName Win32_Processor -ErrorAction Stop
} catch {}
# Intel hybrid hint: use first physical core count logical CPUs as P-cores when log > phys
$pCount = [int]([Math]::Min($phys, $log))
0..($log-1) | ForEach-Object {
  if ($_ -lt $pCount) { $perf += $_ } else { $eff += $_ }
}
@{ performanceCores = $perf; efficientCores = $eff; note = "Heuristic P-core set: first $pCount logical CPUs (hybrid WMI limited)" } | ConvertTo-Json -Compress
"#;
        let v = run_json(script)?;
        Ok(serde_json::from_value(v)?)
    }
    #[cfg(not(windows))]
    {
        Ok(CoreTopology {
            performance_cores: vec![0, 1],
            efficient_cores: vec![],
            note: "Demo topology".into(),
        })
    }
}

pub fn load_watcher_config() -> AppResult<GameWatcherConfig> {
    let p = crate::backup::data_dir()?.join("game_watcher.json");
    if !p.exists() {
        return Ok(GameWatcherConfig::default());
    }
    Ok(serde_json::from_str(&std::fs::read_to_string(p)?)?)
}

pub fn save_watcher_config(cfg: &GameWatcherConfig) -> AppResult<()> {
    let p = crate::backup::data_dir()?.join("game_watcher.json");
    std::fs::write(p, serde_json::to_string_pretty(cfg)?)?;
    Ok(())
}

pub fn apply_ifeo_priority(exe_name: &str, priority: &str) -> AppResult<()> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let class = match priority {
            "high" => 3u32,
            "above_normal" => 2,
            _ => 2,
        };
        let hk = RegKey::predef(HKEY_LOCAL_MACHINE);
        let path = format!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Image File Execution Options\\{exe_name}\\PerfOptions");
        let (key, _) = hk.create_subkey(&path)?;
        key.set_value("CpuPriorityClass", &class)?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = (exe_name, priority);
        Ok(())
    }
}

pub fn set_process_affinity_for_pid(pid: u32, cores: &[u32]) -> AppResult<()> {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_SET_INFORMATION, SetProcessAffinityMask};

        let mask: usize = cores.iter().fold(0usize, |acc, c| acc | (1usize << *c));
        unsafe {
            let h = OpenProcess(PROCESS_SET_INFORMATION, false, pid)
                .map_err(|e| AppError::msg(e.to_string()))?;
            if h.is_invalid() {
                return Err(AppError::msg("OpenProcess failed"));
            }
            SetProcessAffinityMask(h, mask).map_err(|e| AppError::msg(e.to_string()))?;
            let _ = CloseHandle(h).map_err(|e| AppError::msg(e.to_string()));
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = (pid, cores);
        Ok(())
    }
}

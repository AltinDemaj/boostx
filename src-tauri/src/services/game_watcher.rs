use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::hardware::{detect_core_topology, load_watcher_config, save_watcher_config, GameWatcherConfig};

static RUNNING: AtomicBool = AtomicBool::new(false);
static LAST_CONFIG: Mutex<Option<GameWatcherConfig>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatcherStatus {
    pub enabled: bool,
    pub process_names: Vec<String>,
    pub last_tick: Option<String>,
    pub note: String,
}

pub fn watcher_status() -> AppResult<WatcherStatus> {
    let cfg = load_watcher_config()?;
    Ok(WatcherStatus {
        enabled: RUNNING.load(Ordering::SeqCst) && cfg.enabled,
        process_names: cfg.process_names.clone(),
        last_tick: None,
        note: if RUNNING.load(Ordering::SeqCst) {
            "Polling for game processes".into()
        } else {
            "Stopped".into()
        },
    })
}

pub fn start_game_watcher(config: GameWatcherConfig) -> AppResult<()> {
    save_watcher_config(&config)?;
    if !config.enabled {
        stop_game_watcher()?;
        return Ok(());
    }
    if RUNNING.swap(true, Ordering::SeqCst) {
        if let Ok(mut guard) = LAST_CONFIG.lock() {
            *guard = Some(config);
        }
        return Ok(());
    }
    if let Ok(mut guard) = LAST_CONFIG.lock() {
        *guard = Some(config.clone());
    }

    thread::spawn(|| {
        let topology = detect_core_topology().unwrap_or_default();
        let perf = topology.performance_cores.clone();
        while RUNNING.load(Ordering::SeqCst) {
            let cfg = LAST_CONFIG
                .lock()
                .ok()
                .and_then(|g| g.clone())
                .or_else(|| load_watcher_config().ok())
                .unwrap_or_default();
            if !cfg.enabled {
                break;
            }
            #[cfg(windows)]
            {
                let script = format!(
                    r#"
$names = @({})
foreach ($n in $names) {{
  Get-Process -Name $n -ErrorAction SilentlyContinue | ForEach-Object {{ $_.Id }}
}}
"#,
                    cfg.process_names
                        .iter()
                        .map(|n| format!("'{}'", n.replace('\'', "''")))
                        .collect::<Vec<_>>()
                        .join(",")
                );
                if let Ok(out) = crate::platform::powershell::run(&script) {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    for line in stdout.lines() {
                        if let Ok(pid) = line.trim().parse::<u32>() {
                            if cfg.use_performance_cores_only && !perf.is_empty() {
                                let _ = crate::hardware::set_process_affinity_for_pid(pid, &perf);
                            }
                        }
                    }
                }
            }
            thread::sleep(Duration::from_secs(2));
        }
        RUNNING.store(false, Ordering::SeqCst);
    });
    Ok(())
}

pub fn stop_game_watcher() -> AppResult<()> {
    RUNNING.store(false, Ordering::SeqCst);
    let mut cfg = load_watcher_config()?;
    cfg.enabled = false;
    save_watcher_config(&cfg)?;
    Ok(())
}

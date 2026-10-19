#[cfg(windows)]
mod powershell_scan;
mod affinity;
mod boost;
mod gpu_telemetry;
mod rebar;
mod scan;
mod score;

pub use affinity::{
    apply_ifeo_priority, detect_core_topology, load_watcher_config, save_watcher_config,
    set_process_affinity_for_pid, CoreTopology, GameWatcherConfig,
};
pub use boost::{
    apply_safe_gpu_boost, read_nvidia_power_limits, revert_gpu_boost, start_boost_temp_guard,
    stop_boost_temp_guard, BoostSession, GpuPowerLimits,
};
pub use gpu_telemetry::{sample_nvidia, GpuTelemetrySample};
pub use rebar::{detect_rebar, RebarDetection, RebarStatus};
pub use scan::{scan_system, SystemReport};
pub use score::compute_performance_score;

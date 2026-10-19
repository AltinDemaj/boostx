mod admin;
mod auto_optimize;
mod backup;
mod benchmark;
mod bios;
mod error;
mod games;
mod hardware;
mod platform;
pub mod selftest;
mod services;
mod tweaks;

pub const SELFTEST_APP_VERSION: &str = "0.5.0";

use backup::{load_manifest, revert_all};
use benchmark::{
    get_benchmark_history, run_competitive_benchmark, run_quick_benchmark, BenchmarkComparison,
    BenchmarkHistory, BenchmarkPhaseInput, BenchmarkSnapshot, Cs2CompetitiveResult,
};
use bios::{build_interactive, InteractiveBiosReport};
use games::{build_cs2_plan, detect_installed_games, Cs2Optimization, InstalledGame};
use hardware::{
    apply_ifeo_priority, apply_safe_gpu_boost, detect_core_topology, read_nvidia_power_limits,
    revert_gpu_boost, sample_nvidia, scan_system, BoostSession, CoreTopology, GameWatcherConfig,
    GpuPowerLimits, GpuTelemetrySample, SystemReport,
};
use services::{
    measure_timer_resolution_us, set_timer_policy, start_game_watcher, start_timer_service,
    stop_game_watcher, stop_timer_service, watcher_status, TimerPolicy, WatcherStatus,
};
use tauri::{AppHandle, Emitter};
use tweaks::{
    apply_custom_profile, apply_profile, confirm_session, delete_profile, evaluate_post_benchmark,
    evaluate_post_score,
    get_catalog, list_tweak_states, load_profiles, load_safe_mode, revert_single_tweak,
    rename_profile, toggle_tweak, upsert_profile, ApplyProfileResult, CustomProfile,
    CustomProfileStore,
    OptimizationProfile, ToggleTweakResult, TweakCatalogEntry,
};

#[tauri::command]
fn get_elevation_status() -> admin::ElevationStatus {
    admin::elevation_status()
}

#[tauri::command]
fn request_elevation() -> Result<bool, error::AppError> {
    admin::request_elevation_restart().map_err(error::AppError::msg)
}

#[tauri::command]
fn run_tweak_diagnostics(report: SystemReport) -> tweaks::DiagnosticsReport {
    tweaks::run_diagnostics(&report)
}

#[tauri::command]
fn scan_hardware() -> Result<SystemReport, error::AppError> {
    scan_system()
}

#[tauri::command]
fn get_tweak_catalog(report: SystemReport) -> Result<Vec<TweakCatalogEntry>, error::AppError> {
    get_catalog(&report)
}

#[tauri::command]
fn list_tweaks(
    report: SystemReport,
) -> Result<Vec<(tweaks::TweakDefinition, tweaks::TweakState)>, error::AppError> {
    list_tweak_states(&report)
}

#[tauri::command]
fn toggle_tweak_cmd(
    tweak_id: String,
    enable: bool,
    report: SystemReport,
) -> Result<ToggleTweakResult, error::AppError> {
    toggle_tweak(&tweak_id, enable, &report)
}

#[tauri::command]
fn apply_optimization_profile(
    profile: OptimizationProfile,
    report: SystemReport,
) -> Result<ApplyProfileResult, error::AppError> {
    apply_profile(profile, &report)
}

#[tauri::command]
fn apply_custom_profile_cmd(
    profile_id: String,
    report: SystemReport,
) -> Result<ApplyProfileResult, error::AppError> {
    apply_custom_profile(&profile_id, &report)
}

#[tauri::command]
fn save_custom_profile(name: String, tweak_ids: Vec<String>) -> Result<CustomProfile, error::AppError> {
    upsert_profile(name, tweak_ids)
}

#[tauri::command]
fn get_custom_profiles() -> Result<CustomProfileStore, error::AppError> {
    load_profiles()
}

#[tauri::command]
fn delete_custom_profile(profile_id: String) -> Result<(), error::AppError> {
    delete_profile(&profile_id)
}

#[tauri::command]
fn rename_custom_profile(profile_id: String, name: String) -> Result<CustomProfile, error::AppError> {
    rename_profile(&profile_id, name)
}

#[tauri::command]
fn revert_tweak(tweak_id: String) -> Result<(), error::AppError> {
    revert_single_tweak(&tweak_id)
}

#[tauri::command]
fn revert_all_tweaks() -> Result<u32, error::AppError> {
    revert_all()
}

#[tauri::command]
fn get_backup_manifest() -> Result<backup::BackupManifest, error::AppError> {
    load_manifest()
}

#[tauri::command]
fn list_games() -> Result<Vec<InstalledGame>, error::AppError> {
    detect_installed_games()
}

#[tauri::command]
fn get_cs2_plan(report: SystemReport) -> Result<Cs2Optimization, error::AppError> {
    build_cs2_plan(&report)
}

#[tauri::command]
fn get_bios_checklist(report: SystemReport) -> bios::BiosAssistantReport {
    bios::build_bios_checklist(&report)
}

#[tauri::command]
fn get_interactive_bios(report: SystemReport) -> InteractiveBiosReport {
    build_interactive(&report)
}

#[tauri::command]
fn reboot_to_bios(report: SystemReport) -> Result<(), error::AppError> {
    bios::mark_reboot_pending(&report)?;
    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("shutdown")
            .args(["/r", "/fw", "/t", "0"])
            .spawn()
            .map_err(|e| error::AppError::msg(e.to_string()))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Err(error::AppError::msg(
            "Reboot to firmware is only available on Windows.",
        ))
    }
}

#[tauri::command]
fn run_benchmark() -> Result<BenchmarkSnapshot, error::AppError> {
    run_quick_benchmark()
}

#[tauri::command]
fn run_cs2_competitive_benchmark(
    app: AppHandle,
    phase: BenchmarkPhaseInput,
    label: String,
    active_tweak_ids: Vec<String>,
    bios_summary: String,
) -> Result<Cs2CompetitiveResult, error::AppError> {
    run_competitive_benchmark(
        Some(app),
        phase,
        label,
        active_tweak_ids,
        bios_summary,
    )
}

#[tauri::command]
fn get_core_topology() -> Result<CoreTopology, error::AppError> {
    detect_core_topology()
}

#[tauri::command]
fn get_game_watcher_status() -> Result<WatcherStatus, error::AppError> {
    watcher_status()
}

#[tauri::command]
fn configure_game_watcher(config: GameWatcherConfig) -> Result<(), error::AppError> {
    if config.enabled {
        for exe in &config.process_names {
            apply_ifeo_priority(exe, &config.priority_class)?;
        }
    }
    start_game_watcher(config)
}

#[tauri::command]
fn stop_game_watcher_cmd() -> Result<(), error::AppError> {
    stop_game_watcher()
}

#[tauri::command]
fn get_timer_resolution_us() -> Result<f64, error::AppError> {
    measure_timer_resolution_us()
}

#[tauri::command]
fn configure_timer_policy(policy: TimerPolicy) -> Result<(), error::AppError> {
    set_timer_policy(&policy)
}

#[tauri::command]
fn start_timer_service_cmd(policy: TimerPolicy) -> Result<(), error::AppError> {
    start_timer_service(policy)
}

#[tauri::command]
fn stop_timer_service_cmd() -> Result<(), error::AppError> {
    stop_timer_service()
}

#[tauri::command]
fn get_gpu_power_limits() -> Result<GpuPowerLimits, error::AppError> {
    read_nvidia_power_limits()
}

#[tauri::command]
fn apply_gpu_boost(watts: u32) -> Result<BoostSession, error::AppError> {
    apply_safe_gpu_boost(watts)
}

#[tauri::command]
fn revert_gpu_boost_cmd() -> Result<(), error::AppError> {
    revert_gpu_boost()
}

#[tauri::command]
fn get_gpu_telemetry_sample() -> Result<GpuTelemetrySample, error::AppError> {
    sample_nvidia()
}

#[tauri::command]
fn fetch_starting_score() -> Result<Option<benchmark::BenchmarkHistoryEntry>, error::AppError> {
    benchmark::load_starting_score()
}

#[tauri::command]
fn fetch_benchmark_pipeline_log() -> Result<Option<benchmark::BenchmarkPipelineLog>, error::AppError> {
    benchmark::load_benchmark_pipeline_log()
}

#[tauri::command]
fn fetch_benchmark_history() -> Result<BenchmarkHistory, error::AppError> {
    get_benchmark_history()
}

#[tauri::command]
fn compare_benchmarks(
    before: Option<BenchmarkSnapshot>,
    after: Option<BenchmarkSnapshot>,
) -> BenchmarkComparison {
    benchmark::compare(before, after)
}

#[tauri::command]
fn safe_mode_status() -> Result<Option<tweaks::safe_mode::SafeModeSession>, error::AppError> {
    load_safe_mode()
}

#[tauri::command]
fn safe_mode_confirm() -> Result<(), error::AppError> {
    confirm_session()
}

#[tauri::command]
fn safe_mode_evaluate(score: u32) -> Result<Option<String>, error::AppError> {
    evaluate_post_score(score)
}

#[tauri::command]
fn safe_mode_evaluate_benchmark(avg_fps: f32, p1_fps: f32) -> Result<Option<String>, error::AppError> {
    evaluate_post_benchmark(avg_fps, p1_fps)
}

#[tauri::command]
fn start_auto_optimize(app: tauri::AppHandle, report: SystemReport) -> Result<(), error::AppError> {
    if !admin::elevation_status().is_elevated {
        return Err(error::AppError::msg(
            "Administrator rights required for Auto Optimize.",
        ));
    }
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = auto_optimize::run_auto_optimize(handle.clone(), report) {
            let _ = handle.emit("auto-optimize-error", e.to_string());
        }
    });
    Ok(())
}

#[tauri::command]
fn cancel_auto_optimize_cmd() -> Result<(), error::AppError> {
    auto_optimize::cancel_auto_optimize()
}

#[tauri::command]
fn fetch_auto_optimize_report() -> Result<Option<auto_optimize::AutoOptimizeReport>, error::AppError> {
    auto_optimize::load_last_report()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if admin::should_prompt_elevation_on_start() {
        if admin::request_elevation_restart().unwrap_or(false) {
            return;
        }
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_elevation_status,
            request_elevation,
            run_tweak_diagnostics,
            scan_hardware,
            get_tweak_catalog,
            list_tweaks,
            toggle_tweak_cmd,
            apply_optimization_profile,
            apply_custom_profile_cmd,
            save_custom_profile,
            get_custom_profiles,
            delete_custom_profile,
            rename_custom_profile,
            revert_tweak,
            revert_all_tweaks,
            get_backup_manifest,
            list_games,
            get_cs2_plan,
            get_bios_checklist,
            get_interactive_bios,
            reboot_to_bios,
            run_benchmark,
            run_cs2_competitive_benchmark,
            fetch_benchmark_history,
            compare_benchmarks,
            safe_mode_status,
            safe_mode_confirm,
            safe_mode_evaluate,
            safe_mode_evaluate_benchmark,
            get_core_topology,
            get_game_watcher_status,
            configure_game_watcher,
            stop_game_watcher_cmd,
            get_timer_resolution_us,
            configure_timer_policy,
            start_timer_service_cmd,
            stop_timer_service_cmd,
            get_gpu_power_limits,
            apply_gpu_boost,
            revert_gpu_boost_cmd,
            get_gpu_telemetry_sample,
            fetch_starting_score,
            fetch_benchmark_pipeline_log,
            start_auto_optimize,
            cancel_auto_optimize_cmd,
            fetch_auto_optimize_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

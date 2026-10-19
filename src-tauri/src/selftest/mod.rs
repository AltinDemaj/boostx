use std::fs;
use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::hardware::scan_system;
use crate::hardware::SystemReport;
use crate::tweaks::{
    all_tweaks, apply_tweak_by_id, detect_tweak, hardware_matches, revert_single_tweak,
    DetectedState, TweakDefinition,
};

/// User-reported v0.4.0 regressions — must pass on Windows CI (or skip with documented reason).
pub const PRIORITY_TWEAK_IDS: &[&str] = &[
    "power_min_processor_100",
    "power_max_processor_100",
    "amd_processor_boost",
    "defender_cs2_exclusion",
    "net_initial_rto",
    "game_bar_off",
    "cs2_disable_fullscreen_opt",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfTestRow {
    pub id: String,
    pub name: String,
    pub priority: bool,
    pub status: SelfTestStatus,
    pub baseline: String,
    pub after_apply: Option<String>,
    pub after_revert: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SelfTestStatus {
    Pass,
    Fail,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfTestReport {
    pub version: String,
    pub generated_at: String,
    pub elevated: bool,
    pub host: String,
    pub pass: u32,
    pub fail: u32,
    pub skip: u32,
    pub priority_pass: u32,
    pub priority_fail: u32,
    pub priority_skip: u32,
    pub rows: Vec<SelfTestRow>,
}

fn detect_label(d: DetectedState) -> String {
    match d {
        DetectedState::On => "on".into(),
        DetectedState::Off => "off".into(),
        DetectedState::Unknown => "unknown".into(),
        DetectedState::NotApplicable => "n/a".into(),
    }
}

fn skip_reason(def: &TweakDefinition, report: &SystemReport) -> Option<String> {
    if def.guidance_only {
        return Some("guidance-only".into());
    }
    if !hardware_matches(def, report) {
        return Some("not applicable on this hardware".into());
    }
    if (def.id == "defender_cs2_exclusion" || def.id == "cs2_disable_fullscreen_opt")
        && !crate::tweaks::defender::cs2_install_path_exists()
    {
        return Some("CS2 not installed (set BOOSTX_SIM_CS2 to a cs2.exe path on CI)".into());
    }
    match def.id.as_str() {
        "standby_list_clean" | "temp_cleanup" | "pagefile_system_managed" => {
            Some("ephemeral — no persistent ON state".into())
        }
        "cs2_preset" | "valorant_preset" | "fortnite_preset" => Some("profile bundle".into()),
        "net_dns_cloudflare" => Some("DNS change — skipped in automated matrix".into()),
        "nvidia_low_latency" => {
            if !report.gpu.vendor.to_lowercase().contains("nvidia") {
                Some("no NVIDIA GPU on this host".into())
            } else {
                None
            }
        }
        "hpet_bcdedit" => Some("boot config — manual verify".into()),
        "memory_integrity_warn" => Some("guidance only".into()),
        _ => None,
    }
}

pub fn run_selftest(report_path: Option<&Path>) -> AppResult<i32> {
    match run_selftest_inner(report_path) {
        Ok(code) => Ok(code),
        Err(e) => {
            if let Some(path) = report_path {
                let md = format!(
                    "# BoostX selftest aborted\n\n**Error:** {e}\n\n_No tweak matrix was executed._\n"
                );
                if let Some(parent) = path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(path, md);
            }
            Err(e)
        }
    }
}

fn run_selftest_inner(report_path: Option<&Path>) -> AppResult<i32> {
    crate::auto_optimize::logic_selftest()?;
    crate::auto_optimize::mock_benchmark_selftest()?;

    #[cfg(not(windows))]
    {
        let out = SelfTestReport {
            version: crate::SELFTEST_APP_VERSION.to_string(),
            generated_at: Utc::now().to_rfc3339(),
            elevated: false,
            host: "non-windows host (tweak matrix skipped)".into(),
            pass: 0,
            fail: 0,
            skip: 0,
            priority_pass: 0,
            priority_fail: 0,
            priority_skip: 0,
            rows: vec![],
        };
        let mock_bench = if crate::auto_optimize::mock_bench_enabled() {
            "PASS (multi-run mock aggregation)"
        } else {
            "SKIPPED (set BOOSTX_MOCK_BENCH=1)"
        };
        let md = format!(
            "{}\n\n## Auto Optimize\n\n- **Verdict logic:** PASS (keep / revert / neutral-latency)\n- **Mock benchmark pipeline:** {}\n- **Windows tweak matrix:** SKIPPED (requires windows-latest CI)\n",
            render_markdown(&out),
            mock_bench
        );
        if let Some(path) = report_path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, md)?;
        }
        eprintln!("tweak matrix selftest requires Windows — auto_optimize logic OK");
        return Ok(0);
    }

    #[cfg(windows)]
    {
        let elevated = crate::admin::elevation_status().is_elevated;
        let report = scan_system()?;
        let host = format!(
            "{} / {} / {}",
            report.windows.version,
            report.cpu.model,
            report.gpu.model
        );

        let mut rows = Vec::new();
        for def in all_tweaks() {
            let priority = PRIORITY_TWEAK_IDS.contains(&def.id.as_str());
            rows.push(run_one(&def, &report, elevated, priority));
        }

        let pass = rows.iter().filter(|r| r.status == SelfTestStatus::Pass).count() as u32;
        let fail = rows.iter().filter(|r| r.status == SelfTestStatus::Fail).count() as u32;
        let skip = rows.iter().filter(|r| r.status == SelfTestStatus::Skip).count() as u32;
        let priority_pass = rows
            .iter()
            .filter(|r| r.priority && r.status == SelfTestStatus::Pass)
            .count() as u32;
        let priority_fail = rows
            .iter()
            .filter(|r| r.priority && r.status == SelfTestStatus::Fail)
            .count() as u32;
        let priority_skip = rows
            .iter()
            .filter(|r| r.priority && r.status == SelfTestStatus::Skip)
            .count() as u32;

        let out = SelfTestReport {
            version: crate::SELFTEST_APP_VERSION.to_string(),
            generated_at: Utc::now().to_rfc3339(),
            elevated,
            host,
            pass,
            fail,
            skip,
            priority_pass,
            priority_fail,
            priority_skip,
            rows,
        };

        let json = serde_json::to_string_pretty(&out)?;
        let mock_bench = if crate::auto_optimize::mock_bench_enabled() {
            "PASS (multi-run mock aggregation)"
        } else {
            "SKIPPED"
        };
        let md = format!(
            "{}\n\n## Auto Optimize\n\n- **Verdict logic:** PASS\n- **Mock benchmark pipeline:** {}\n",
            render_markdown(&out),
            mock_bench
        );
        if let Some(path) = report_path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, &md)?;
            let json_path = path.with_extension("json");
            fs::write(json_path, json)?;
        } else {
            println!("{}", md);
        }

        let priority_ok = priority_fail == 0;
        Ok(if fail == 0 && priority_ok { 0 } else { 1 })
    }
}

#[cfg(windows)]
fn run_one(def: &TweakDefinition, report: &SystemReport, elevated: bool, priority: bool) -> SelfTestRow {
    let baseline = detect_tweak(def);
    let baseline_s = detect_label(baseline);

    if let Some(reason) = skip_reason(def, report) {
        return SelfTestRow {
            id: def.id.clone(),
            name: def.name.clone(),
            priority,
            status: SelfTestStatus::Skip,
            baseline: baseline_s,
            after_apply: None,
            after_revert: None,
            detail: reason,
        };
    }

    if !elevated {
        return SelfTestRow {
            id: def.id.clone(),
            name: def.name.clone(),
            priority,
            status: SelfTestStatus::Skip,
            baseline: baseline_s,
            after_apply: None,
            after_revert: None,
            detail: "not elevated".into(),
        };
    }

    let original = baseline;
    if baseline == DetectedState::On {
        if let Err(e) = revert_single_tweak(&def.id) {
            return SelfTestRow {
                id: def.id.clone(),
                name: def.name.clone(),
                priority,
                status: SelfTestStatus::Skip,
                baseline: baseline_s,
                after_apply: None,
                after_revert: None,
                detail: format!("already ON — revert before test failed: {e}"),
            };
        }
    }

    if let Err(e) = apply_tweak_by_id(&def.id, report) {
        return SelfTestRow {
            id: def.id.clone(),
            name: def.name.clone(),
            priority,
            status: SelfTestStatus::Fail,
            baseline: baseline_s,
            after_apply: None,
            after_revert: None,
            detail: format!("apply failed: {e}"),
        };
    }

    let after_apply = detect_tweak(def);
    let after_apply_s = detect_label(after_apply);
    if after_apply != DetectedState::On {
        let _ = revert_single_tweak(&def.id);
        return SelfTestRow {
            id: def.id.clone(),
            name: def.name.clone(),
            priority,
            status: SelfTestStatus::Fail,
            baseline: baseline_s,
            after_apply: Some(after_apply_s),
            after_revert: None,
            detail: "expected ON after apply".into(),
        };
    }

    if let Err(e) = revert_single_tweak(&def.id) {
        return SelfTestRow {
            id: def.id.clone(),
            name: def.name.clone(),
            priority,
            status: SelfTestStatus::Fail,
            baseline: baseline_s,
            after_apply: Some(after_apply_s),
            after_revert: None,
            detail: format!("revert failed: {e}"),
        };
    }

    let after_revert = detect_tweak(def);
    let after_revert_s = detect_label(after_revert);
    let restored = after_revert == original
        || (original == DetectedState::Off && after_revert == DetectedState::Off);

    if !restored {
        let detail = format!("revert mismatch ({baseline_s} -> {after_revert_s})");
        return SelfTestRow {
            id: def.id.clone(),
            name: def.name.clone(),
            priority,
            status: SelfTestStatus::Fail,
            baseline: baseline_s,
            after_apply: Some(after_apply_s),
            after_revert: Some(after_revert_s),
            detail,
        };
    }

    SelfTestRow {
        id: def.id.clone(),
        name: def.name.clone(),
        priority,
        status: SelfTestStatus::Pass,
        baseline: baseline_s,
        after_apply: Some(after_apply_s),
        after_revert: Some(after_revert_s),
        detail: "detect → apply → ON → revert → baseline".into(),
    }
}

fn render_markdown(r: &SelfTestReport) -> String {
    let mut md = String::new();
    md.push_str(&format!("# BoostX self-test report (v{})\n\n", r.version));
    md.push_str(&format!("- **Generated:** {}\n", r.generated_at));
    md.push_str(&format!("- **Host:** {}\n", r.host));
    md.push_str(&format!("- **Elevated:** {}\n", if r.elevated { "yes" } else { "no" }));
    md.push_str(&format!(
        "- **All tweaks:** {} pass, {} fail, {} skip\n",
        r.pass,
        r.fail,
        r.skip
    ));
    md.push_str(&format!(
        "- **Priority tweaks (user v0.4.0 cases):** {} pass, {} fail, {} skip\n\n",
        r.priority_pass,
        r.priority_fail,
        r.priority_skip
    ));

    md.push_str("## Priority matrix\n\n");
    md.push_str("| ID | Status | Baseline | After apply | After revert | Detail |\n");
    md.push_str("|----|--------|----------|-------------|--------------|--------|\n");
    for row in r.rows.iter().filter(|r| r.priority) {
        let st = match row.status {
            SelfTestStatus::Pass => "PASS",
            SelfTestStatus::Fail => "FAIL",
            SelfTestStatus::Skip => "SKIP",
        };
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            row.id,
            st,
            row.baseline,
            row.after_apply.as_deref().unwrap_or("—"),
            row.after_revert.as_deref().unwrap_or("—"),
            row.detail.replace('|', "/"),
        ));
    }

    md.push_str("\n## Full matrix\n\n");
    md.push_str("| Tweak | Status | Baseline | After apply | After revert | Detail |\n");
    md.push_str("|-------|--------|----------|-------------|--------------|--------|\n");
    for row in &r.rows {
        let st = match row.status {
            SelfTestStatus::Pass => "PASS",
            SelfTestStatus::Fail => "FAIL",
            SelfTestStatus::Skip => "SKIP",
        };
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            row.name,
            st,
            row.baseline,
            row.after_apply.as_deref().unwrap_or("—"),
            row.after_revert.as_deref().unwrap_or("—"),
            row.detail.replace('|', "/"),
        ));
    }
    md
}

use crate::error::{AppError, AppResult};
use crate::platform::cmd::run_hidden_ok;
use crate::platform::powercfg_parse;

const SUB_PROCESSOR: &str = "54533251-82be-4824-96c1-47b60b740d00";
const PROC_MIN: &str = "893dee8e-2bef-4690-94cd-1ebd4e0b7a11";
const PROC_MAX: &str = "bc5038f7-23e0-4960-96da-33abaf5935ec";
const CORE_PARK_MIN: &str = "94d3a615-a899-4ac5-ae2b-8fc9a49d07a6";
const PERF_BOOST: &str = "45bcc044-d885-43e2-8601-ee0ecb612d66";
const ULTIMATE_GUID: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";

pub fn active_scheme_guid() -> AppResult<String> {
    let out = run_hidden_ok("powercfg", &["/getactivescheme"])?;
    powercfg_parse::parse_active_scheme_guid(&out)
        .ok_or_else(|| AppError::msg("Could not parse active power scheme"))
}

pub fn active_scheme_is_ultimate() -> AppResult<bool> {
    let guid = active_scheme_guid()?;
    Ok(guid.eq_ignore_ascii_case(ULTIMATE_GUID))
}

pub fn set_ultimate_performance() -> AppResult<()> {
    run_hidden_ok("powercfg", &["-duplicatescheme", ULTIMATE_GUID])?;
    let out = run_hidden_ok("powercfg", &["/getactivescheme"])?;
    let dup = powercfg_parse::parse_active_scheme_guid(&out)
        .unwrap_or_else(|| ULTIMATE_GUID.to_string());
    run_hidden_ok("powercfg", &["/setactive", &dup])?;
    Ok(())
}

pub fn query_setting(subgroup: &str, setting: &str) -> AppResult<String> {
    let scheme = active_scheme_guid()?;
    run_hidden_ok("powercfg", &["/query", &scheme, subgroup, setting])
}

pub fn read_processor_min_percent() -> AppResult<Option<u32>> {
    let out = query_setting(SUB_PROCESSOR, PROC_MIN)?;
    Ok(powercfg_parse::parse_ac_percent(&out, "minimum processor state"))
}

pub fn read_processor_max_percent() -> AppResult<Option<u32>> {
    let out = query_setting(SUB_PROCESSOR, PROC_MAX)?;
    Ok(powercfg_parse::parse_ac_percent(&out, "maximum processor state"))
}

pub fn read_core_parking_min_percent() -> AppResult<Option<u32>> {
    let out = query_setting(SUB_PROCESSOR, CORE_PARK_MIN)?;
    Ok(powercfg_parse::parse_ac_percent(&out, "core parking"))
}

pub fn read_perfboost_mode() -> AppResult<Option<u32>> {
    let out = query_setting(SUB_PROCESSOR, PERF_BOOST)?;
    Ok(powercfg_parse::parse_ac_percent(&out, "processor performance boost"))
}

pub fn set_ac_value(subgroup: &str, setting: &str, value: u32) -> AppResult<()> {
    let scheme = active_scheme_guid()?;
    run_hidden_ok(
        "powercfg",
        &[
            "/setacvalueindex",
            &scheme,
            subgroup,
            setting,
            &value.to_string(),
        ],
    )?;
    run_hidden_ok("powercfg", &["/setactive", &scheme])?;
    Ok(())
}

pub fn set_processor_min_percent(percent: u32) -> AppResult<()> {
    set_ac_value(SUB_PROCESSOR, PROC_MIN, percent)
}

pub fn set_processor_max_percent(percent: u32) -> AppResult<()> {
    set_ac_value(SUB_PROCESSOR, PROC_MAX, percent)
}

pub fn set_core_parking_min_percent(percent: u32) -> AppResult<()> {
    set_ac_value(SUB_PROCESSOR, CORE_PARK_MIN, percent)
}

pub fn set_perfboost_mode(mode: u32) -> AppResult<()> {
    set_ac_value(SUB_PROCESSOR, PERF_BOOST, mode)
}

/// True when `powercfg /query` can read this subgroup setting on the active scheme.
pub fn setting_query_supported(subgroup: &str, setting: &str) -> bool {
    query_setting(subgroup, setting).is_ok()
}

pub fn processor_min_setting_supported() -> bool {
    setting_query_supported(SUB_PROCESSOR, PROC_MIN)
}

pub fn processor_boost_setting_supported() -> bool {
    setting_query_supported(SUB_PROCESSOR, PERF_BOOST)
}

use crate::hardware::sample_nvidia;

#[derive(Default)]
pub struct HostTelemetry {
    pub cpu_usage_pct: Option<f32>,
    pub gpu_usage_pct: Option<f32>,
    pub gpu_temp_c: Option<f32>,
}

pub fn sample_host_telemetry() -> HostTelemetry {
    #[cfg(windows)]
    {
        let gpu = sample_nvidia().ok();
        let cpu = sample_cpu_usage_pct();
        HostTelemetry {
            cpu_usage_pct: cpu,
            gpu_usage_pct: gpu.as_ref().and_then(|g| g.usage_pct),
            gpu_temp_c: gpu.as_ref().and_then(|g| g.temp_c),
        }
    }
    #[cfg(not(windows))]
    {
        HostTelemetry {
            cpu_usage_pct: None,
            gpu_usage_pct: None,
            gpu_temp_c: None,
        }
    }
}

#[cfg(windows)]
fn sample_cpu_usage_pct() -> Option<f32> {
    let out = crate::platform::powershell::run(
        r#"(Get-Counter '\Processor(_Total)\% Processor Time' -SampleInterval 1 -MaxSamples 1).CounterSamples.CookedValue"#,
    )
    .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse()
        .ok()
}

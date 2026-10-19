use chrono::Utc;

use crate::error::AppResult;
use crate::hardware::scan::{
    CpuInfo, GpuInfo, MonitorInfo, MotherboardInfo, RamInfo, StorageInfo, SystemReport, WindowsInfo,
};
use crate::hardware::score::compute_performance_score;
use crate::platform::powershell::run_json;

pub fn scan_windows() -> AppResult<SystemReport> {
    let script = r#"
$cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
$gpu = Get-CimInstance Win32_VideoController | Where-Object { $_.Name -notmatch 'Microsoft Basic' } | Select-Object -First 1
$cs = Get-CimInstance Win32_ComputerSystem
$os = Get-CimInstance Win32_OperatingSystem
$board = Get-CimInstance Win32_BaseBoard | Select-Object -First 1
$bios = Get-CimInstance Win32_BIOS | Select-Object -First 1
$disks = Get-CimInstance Win32_DiskDrive | Select-Object Model, Size, MediaType
$monitors = @()
try {
  Get-CimInstance -Namespace root/wmi -ClassName WmiMonitorID -ErrorAction Stop | ForEach-Object {
    $name = ($_.UserFriendlyName | ForEach-Object { [char]$_ }) -join ''
    $monitors += @{ name = $name.Trim([char]0); refreshHz = $null; resolution = $null }
  }
} catch {}
if ($monitors.Count -eq 0) { $monitors = @(@{ name = 'Primary'; refreshHz = $null; resolution = $null }) }
$ramSpeed = $null
try { $ramSpeed = (Get-CimInstance Win32_PhysicalMemory | Select-Object -First 1).Speed } catch {}
@{
  cpu = @{
    vendor = $cpu.Manufacturer
    model = $cpu.Name.Trim()
    cores = [uint32]$cpu.NumberOfCores
    logicalProcessors = [uint32]$cpu.NumberOfLogicalProcessors
    generationHint = ''
    baseClockMhz = [uint32]$cpu.MaxClockSpeed
  }
  gpu = if ($gpu) {
    @{
      vendor = if ($gpu.AdapterCompatibility) { $gpu.AdapterCompatibility } else { 'GPU' }
      model = $gpu.Name
      vramMb = if ($gpu.AdapterRAM) { [uint32]($gpu.AdapterRAM / 1MB) } else { $null }
      driverVersion = $gpu.DriverVersion
    }
  } else {
    @{ vendor = 'GPU'; model = 'Unknown'; vramMb = $null; driverVersion = $null }
  }
  ram = @{
    totalGb = [double]($cs.TotalPhysicalMemory / 1GB)
    speedMhz = if ($ramSpeed) { [uint32]$ramSpeed } else { $null }
    ratedSpeedMhz = $null
    xmpProfileDetected = $false
    runningBelowRated = $false
  }
  storage = @($disks | ForEach-Object {
    @{
      model = $_.Model.Trim()
      kind = if ($_.MediaType -match 'SSD|NVMe|Fixed') { 'SSD/NVMe' } else { 'HDD' }
      sizeGb = [double]($_.Size / 1GB)
      freeGb = $null
    }
  })
  motherboard = @{
    manufacturer = $board.Manufacturer.Trim()
    model = $board.Product.Trim()
    biosVersion = $bios.SMBIOSBIOSVersion
    biosVendor = $bios.Manufacturer
  }
  monitors = $monitors
  windows = @{
    version = $os.Caption
    build = $os.BuildNumber
    edition = $os.OSArchitecture
  }
  scanWarnings = @()
} | ConvertTo-Json -Compress -Depth 6
"#;

    let v = run_json(script)?;
    let mut report: SystemReport = serde_json::from_value(v)?;
    if report.cpu.generation_hint.is_empty() {
        report.cpu.generation_hint = guess_cpu_gen(&report.cpu.model);
    }
    if report.cpu.vendor.is_empty() {
        report.cpu.vendor = if report.cpu.model.to_uppercase().contains("AMD") {
            "AMD".into()
        } else {
            "Intel".into()
        };
    }
    if report.gpu.vendor.is_empty() {
        report.gpu.vendor = if report.gpu.model.to_uppercase().contains("NVIDIA") {
            "NVIDIA".into()
        } else if report.gpu.model.to_uppercase().contains("AMD") {
            "AMD".into()
        } else {
            "GPU".into()
        };
    }
    if report.monitors.is_empty() {
        report.monitors.push(MonitorInfo {
            name: "Primary".into(),
            refresh_hz: None,
            resolution: None,
        });
    }
    if report.storage.is_empty() {
        report.storage.push(StorageInfo {
            model: "Unknown".into(),
            kind: "SSD".into(),
            size_gb: 512.0,
            free_gb: None,
        });
    }
    report.scanned_at = Utc::now().to_rfc3339();
    report.is_demo_data = false;
    report.scan_method = "WMI".into();
    let (score, breakdown) = compute_performance_score(&report);
    report.performance_score = score;
    report.score_breakdown = breakdown;
    Ok(report)
}

fn guess_cpu_gen(model: &str) -> String {
    let m = model.to_uppercase();
    if m.contains("14900") || m.contains("13900") {
        "14th/13th Gen".into()
    } else if m.contains("13700") || m.contains("12700") {
        "12th/13th Gen".into()
    } else if m.contains("RYZEN 9") || m.contains("7950") {
        "Zen 4".into()
    } else if m.contains("RYZEN 7") || m.contains("5800") {
        "Zen 3".into()
    } else {
        "Unknown".into()
    }
}

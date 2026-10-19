use super::scan::{ScoreFactor, SystemReport};

pub fn compute_performance_score(report: &SystemReport) -> (u32, Vec<ScoreFactor>) {
    let mut factors = Vec::new();

    let cpu_pts = if report.cpu.logical_processors >= 16 {
        22
    } else if report.cpu.logical_processors >= 8 {
        18
    } else {
        12
    };
    factors.push(ScoreFactor {
        label: "CPU throughput".into(),
        points: cpu_pts,
        max_points: 25,
        note: format!("{} logical CPUs", report.cpu.logical_processors),
    });

    let gpu_pts = if report.gpu.model.to_lowercase().contains("rtx") {
        22
    } else if report.gpu.model.to_lowercase().contains("rx") {
        18
    } else {
        12
    };
    factors.push(ScoreFactor {
        label: "GPU class".into(),
        points: gpu_pts,
        max_points: 25,
        note: report.gpu.model.clone(),
    });

    let ram_pts = if report.ram.total_gb >= 32.0 {
        18
    } else if report.ram.total_gb >= 16.0 {
        14
    } else {
        8
    };
    factors.push(ScoreFactor {
        label: "System RAM".into(),
        points: ram_pts,
        max_points: 20,
        note: format!("{:.0} GB", report.ram.total_gb),
    });

    let storage_pts = if report.storage.iter().any(|d| d.kind.to_lowercase().contains("nvme")) {
        16
    } else {
        10
    };
    factors.push(ScoreFactor {
        label: "Storage".into(),
        points: storage_pts,
        max_points: 15,
        note: report
            .storage
            .first()
            .map(|s| s.model.clone())
            .unwrap_or_else(|| "Unknown".into()),
    });

    let display_pts = report
        .monitors
        .iter()
        .map(|m| m.refresh_hz.unwrap_or(60))
        .max()
        .unwrap_or(60);
    let display_score = if display_pts >= 240 {
        15
    } else if display_pts >= 144 {
        12
    } else {
        8
    };
    factors.push(ScoreFactor {
        label: "Display refresh".into(),
        points: display_score,
        max_points: 15,
        note: format!("{} Hz peak", display_pts),
    });

    let total = factors.iter().map(|f| f.points).sum::<u32>().min(100);
    (total, factors)
}

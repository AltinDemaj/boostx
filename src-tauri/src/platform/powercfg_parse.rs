/// Parse `powercfg /query` AC value lines (0x... index).
pub fn parse_ac_percent(output: &str, setting_alias: &str) -> Option<u32> {
    let needle = setting_alias.to_lowercase();
    let lines: Vec<&str> = output.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if !line.to_lowercase().contains(&needle) {
            continue;
        }
        for j in i..lines.len().min(i + 12) {
            let l = lines[j].to_lowercase();
            if l.contains("current ac power setting index") || l.contains("current dc power setting index") {
                if let Some(hex) = extract_hex_index(lines[j]) {
                    return Some(hex);
                }
            }
        }
    }
    None
}

pub fn parse_scheme_name(output: &str) -> Option<String> {
    for line in output.lines() {
        let t = line.trim();
        if t.starts_with("Power Scheme GUID:") {
            continue;
        }
        if t.starts_with("Power Scheme Name:") {
            return Some(t.split(':').nth(1)?.trim().to_string());
        }
    }
    None
}

pub fn parse_active_scheme_guid(output: &str) -> Option<String> {
    for line in output.lines() {
        let t = line.trim();
        if t.starts_with("Power Scheme GUID:") {
            let rest = t.split(':').nth(1)?.trim();
            let guid = rest
                .trim_matches(|c| c == '(' || c == ')')
                .split_whitespace()
                .next()
                .unwrap_or(rest);
            return Some(guid.to_string());
        }
    }
    None
}

fn extract_hex_index(line: &str) -> Option<u32> {
    let lower = line.to_lowercase();
    if let Some(pos) = lower.find("0x") {
        let hex = line[pos + 2..]
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches(')');
        return u32::from_str_radix(hex, 16).ok();
    }
    None
}

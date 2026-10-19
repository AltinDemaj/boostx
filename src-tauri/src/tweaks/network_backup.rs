use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetTcpBackup {
    pub autotun: Option<String>,
    pub ecn: Option<String>,
    pub rss: Option<String>,
    pub timestamps: Option<String>,
}

pub fn load_backup() -> AppResult<NetTcpBackup> {
    let p = crate::backup::data_dir()?.join("net_tcp_backup.json");
    if !p.exists() {
        return Ok(NetTcpBackup::default());
    }
    Ok(serde_json::from_str(&std::fs::read_to_string(p)?)?)
}

pub fn save_backup(backup: &NetTcpBackup) -> AppResult<()> {
    let p = crate::backup::data_dir()?.join("net_tcp_backup.json");
    std::fs::write(p, serde_json::to_string_pretty(backup)?)?;
    Ok(())
}

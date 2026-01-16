use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
pub struct TraceEvent {
    timestamp_ns: u64,
    duration_ns: u64,
    cmds: Vec<String>,
}

impl TraceEvent {
    pub fn new(timestamp_ns: u64, duration_ns: u64, cmds: Vec<String>) -> Self {
        TraceEvent {
            timestamp_ns,
            duration_ns,
            cmds,
        }
    }

    pub fn from(start: SystemTime, duration: Duration, cmds: Vec<String>) -> anyhow::Result<Self> {
        let epoch = start.duration_since(SystemTime::UNIX_EPOCH)?;
        let timestamp_ns = epoch.as_secs() * 1_000_000_000 + epoch.subsec_nanos() as u64;
        let duration_ns = duration.as_secs() * 1_000_000_000 + duration.subsec_nanos() as u64;
        Ok(TraceEvent::new(timestamp_ns, duration_ns, cmds))
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
    }

    pub fn from_json(json_str: &str) -> Result<Self> {
        serde_json::from_str(json_str)
    }
}

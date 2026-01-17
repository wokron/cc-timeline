use serde::{Deserialize, Serialize};

use crate::trace_event;

#[derive(Serialize, Deserialize)]
struct ChromeTraceEvent {
    name: String,
    cat: String,
    ph: String,
    ts: u64,
    pid: u32,
    tid: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    dur: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
}

pub struct ChromeTraceSaver {
    file: std::fs::File,
    events: Vec<ChromeTraceEvent>,
}

impl ChromeTraceSaver {
    pub fn new(path: &std::path::Path) -> anyhow::Result<Self> {
        let file = std::fs::File::create(path)?;
        Ok(ChromeTraceSaver {
            file,
            events: Vec::new(),
        })
    }

    fn add_event(&mut self, event: ChromeTraceEvent) {
        self.events.push(event);
    }

    pub fn accept_trace_event(&mut self, event: trace_event::TraceEvent) {
        let name = event.cmds.join(" ");
        let program_name = event.cmds.first().cloned().unwrap_or_default();
        let cat = program_name;
        let ph = "X".to_string();
        let ts_us = event.timestamp_ns / 1000;
        let dur_us = event.duration_ns / 1000;
        // Use pid as tid for better visualization grouping
        let tid = event.pid;
        let chrome_event = ChromeTraceEvent {
            name,
            cat,
            ph,
            ts: ts_us,
            pid: 0,
            tid,
            dur: Some(dur_us),
            bp: None,
            id: None,
        };
        self.add_event(chrome_event);
    }

    pub fn accept_flow_event(
        &mut self,
        cat: String,
        from_ns: u64,
        to_ns: u64,
        pid1: u32,
        pid2: u32,
        id: u64,
    ) {
        let start_event = ChromeTraceEvent {
            name: "flow".to_string(),
            cat: cat.clone(),
            ph: "s".to_string(),
            ts: from_ns / 1000,
            pid: 0,
            tid: pid1,
            dur: None,
            id: Some(format!("{}", id)),
            bp: Some("e".to_string()),
        };
        let end_event = ChromeTraceEvent {
            name: "flow".to_string(),
            cat,
            ph: "f".to_string(),
            ts: to_ns / 1000,
            pid: 0,
            tid: pid2,
            dur: None,
            id: Some(format!("{}", id)),
            bp: Some("e".to_string()),
        };
        self.add_event(start_event);
        self.add_event(end_event);
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        serde_json::to_writer(&self.file, &self.events)?;
        Ok(())
    }
}

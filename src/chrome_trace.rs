use serde::{Deserialize, Serialize};

use crate::trace_event;

#[derive(Serialize, Deserialize)]
struct ChromeTraceEvent {
    name: String,
    cat: String,
    ph: String,
    ts: u64,
    dur: u64,
    pid: u32,
}

impl ChromeTraceEvent {
    pub fn new(name: String, cat: String, ph: String, ts: u64, dur: u64, pid: u32) -> Self {
        ChromeTraceEvent {
            name,
            cat,
            ph,
            ts,
            dur,
            pid,
        }
    }
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
        let pid = 0;
        let chrome_event = ChromeTraceEvent::new(name, cat, ph, ts_us, dur_us, pid);
        self.add_event(chrome_event);
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        serde_json::to_writer(&self.file, &self.events)?;
        Ok(())
    }
}

use crate::trace_event::TraceEvent;

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct TraceRecorder {
    file: File,
}

impl TraceRecorder {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(TraceRecorder { file })
    }

    fn record(&mut self, data: &str) -> anyhow::Result<()> {
        writeln!(self.file, "{}", data)?;
        self.file.flush()?;
        Ok(())
    }

    pub fn record_event(&mut self, event: &TraceEvent) -> anyhow::Result<()> {
        let json = event.to_json()?;
        self.record(&json)
    }
}

pub struct TraceLoader {
    file: File,
}

impl TraceLoader {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        Ok(TraceLoader { file })
    }

    pub fn load_events<F>(&mut self, mut cb: F) -> anyhow::Result<()>
    where
        F: FnMut(TraceEvent),
    {
        let reader = BufReader::new(&self.file);
        for line in reader.lines() {
            let line = line?;
            let event = TraceEvent::from_json(&line)?;
            cb(event);
        }
        Ok(())
    }
}

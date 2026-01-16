use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{Duration, SystemTime};

struct Logger {
    file: File,
}

impl Logger {
    fn new(path: &str) -> anyhow::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Logger { file })
    }

    fn write(&mut self, msg: &str) -> anyhow::Result<()> {
        writeln!(self.file, "{}", msg)?;
        self.file.flush()?;
        Ok(())
    }
}

pub trait TimeFormatter {
    fn format_time(&self, ts: SystemTime) -> String;
    fn format_duration(&self, dur: Duration) -> String;
}

pub struct EventLogger<'a> {
    log: Logger,
    time_formatter: &'a dyn TimeFormatter,
}

impl<'a> EventLogger<'a> {
    pub fn new(path: &str, time_formatter: &'a dyn TimeFormatter) -> anyhow::Result<Self> {
        let log = Logger::new(path)?;
        Ok(EventLogger {
            log,
            time_formatter,
        })
    }

    pub fn log_event(&mut self, ts: SystemTime, dur: Duration, ev: &str) -> anyhow::Result<()> {
        let time_str = self.time_formatter.format_time(ts);
        let dur_str = self.time_formatter.format_duration(dur);
        let msg = format!("{} {} {}", time_str, dur_str, ev);
        self.log.write(&msg)
    }
}

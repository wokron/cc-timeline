mod logger;

use crate::logger::{EventLogger, TimeFormatter};
use std::process::ExitStatus;
use std::time::Duration;
use std::time::SystemTime;

use clap::CommandFactory;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Activate debug mode
    #[clap(short, long)]
    debug: bool,

    #[clap(last = true)]
    cmd_args: Vec<String>,
}

fn exec_with_measure(
    program: &str,
    args: &[&String],
) -> anyhow::Result<(ExitStatus, SystemTime, Duration)> {
    let start = std::time::SystemTime::now();
    let status = std::process::Command::new(program).args(args).status()?;
    let end = std::time::SystemTime::now();
    let duration = end.duration_since(start)?;
    Ok((status, start, duration))
}

struct NanoTsTimeFormatter;

impl TimeFormatter for NanoTsTimeFormatter {
    fn format_time(&self, ts: SystemTime) -> String {
        let epoch = ts
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");
        self.format_duration(epoch)
    }

    fn format_duration(&self, dur: Duration) -> String {
        let dur_ns = dur.as_secs() * 1_000_000_000 + dur.subsec_nanos() as u64;
        format!("{}", dur_ns)
    }
}

fn main() {
    let args = Cli::parse();

    if args.debug {
        println!("Debug mode is on");
    } else {
        println!("Debug mode is off");
    }

    if args.cmd_args.is_empty() {
        eprintln!("No additional arguments provided.");
        Cli::command().print_help().unwrap();
        std::process::exit(1);
    }

    let mut cmd_iter = args.cmd_args.iter();
    let program = cmd_iter.next().unwrap(); // Safe
    let program_args: Vec<&String> = cmd_iter.collect();

    let (status, start, duration) =
        exec_with_measure(program, &program_args).expect("Failed to execute the command");
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    let time_formatter = NanoTsTimeFormatter;
    let mut event_logger =
        EventLogger::new("events.log", &time_formatter).expect("Failed to create event logger");
    let cmd_args_str = args.cmd_args.join(" ");
    event_logger
        .log_event(start, duration, cmd_args_str.as_str())
        .expect("Failed to log event");
}

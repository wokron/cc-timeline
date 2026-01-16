mod logger;

use crate::logger::{EventLogger, TimeFormatter};
use std::process::ExitStatus;
use std::time::Duration;
use std::time::SystemTime;

use clap::CommandFactory;
use clap::Parser;
use clap::Subcommand;

#[derive(clap::Args)]
struct TraceArgs {
    #[clap(short, long, default_value = "trace.txt")]
    output: String,

    #[clap(last = true)]
    cmd_args: Vec<String>,
}

#[derive(clap::Args)]
struct ConvertArgs {
    #[clap(short, long, default_value = "trace.txt")]
    input: String,

    #[clap(short, long, default_value = "trace.json")]
    output: String,
}

#[derive(Subcommand)]
enum Commands {
    Trace(TraceArgs),
    Convert(ConvertArgs),
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
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

fn do_trace(args: &TraceArgs) {
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
        EventLogger::new(&args.output, &time_formatter).expect("Failed to create event logger");
    let cmd_args_str = args.cmd_args.join(" ");
    event_logger
        .log_event(start, duration, cmd_args_str.as_str())
        .expect("Failed to log event");
}

fn do_convert(args: &ConvertArgs) {
    eprintln!("Converting from {} to {}", args.input, args.output);
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Trace(trace_args) => do_trace(trace_args),
        Commands::Convert(convert_args) => do_convert(convert_args),
    }
}

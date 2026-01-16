mod chrome_trace;
mod trace;
mod trace_event;

use crate::trace::TraceRecorder;
use std::path::Path;
use std::process::ExitStatus;
use std::time::Duration;
use std::time::SystemTime;

use clap::CommandFactory;
use clap::Parser;
use clap::Subcommand;

#[derive(clap::Args)]
struct TraceArgs {
    #[clap(short, long, default_value = "trace.ndjson")]
    output: String,

    #[clap(last = true)]
    cmd_args: Vec<String>,
}

#[derive(clap::Args)]
struct ConvertArgs {
    #[clap(short, long, default_value = "trace.ndjson")]
    input: String,

    #[clap(short, long, default_value = "chrome_trace.json")]
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

fn do_trace(args: &TraceArgs) {
    if args.cmd_args.is_empty() {
        eprintln!("No additional arguments provided.");
        Cli::command().print_help().unwrap();
        std::process::exit(1);
    }

    let mut cmd_iter = args.cmd_args.iter();
    let program: &String = cmd_iter.next().unwrap(); // Safe
    let program_args: Vec<&String> = cmd_iter.collect();

    let (status, start, duration) =
        exec_with_measure(program, &program_args).expect("Failed to execute the command");
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    let event = trace_event::TraceEvent::from(start, duration, args.cmd_args.clone()).unwrap();
    let mut recorder =
        TraceRecorder::new(Path::new(&args.output)).expect("Failed to create trace recorder");
    recorder
        .record_event(&event)
        .expect("Failed to record trace event");
}

fn do_convert(args: &ConvertArgs) {
    let mut loader =
        trace::TraceLoader::new(Path::new(&args.input)).expect("Failed to create trace loader");
    let mut saver = chrome_trace::ChromeTraceSaver::new(Path::new(&args.output))
        .expect("Failed to create saver");
    loader
        .load_events(|event: trace_event::TraceEvent| saver.accept_trace_event(event))
        .expect("Failed to load trace events");

    saver.save().expect("Failed to save Chrome trace file");
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Trace(trace_args) => do_trace(trace_args),
        Commands::Convert(convert_args) => do_convert(convert_args),
    }
}

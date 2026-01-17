mod chrome_trace;
mod dependency;
mod thread_placer;
mod trace;
mod trace_event;

use crate::chrome_trace::ChromeTraceSaver;
use crate::thread_placer::ThreadPlacer;
use crate::trace::TraceLoader;
use crate::trace::TraceRecorder;
use crate::trace_event::TraceEvent;

use std::path::Path;
use std::process::ExitStatus;
use std::time::Duration;
use std::time::SystemTime;

use clap::CommandFactory;
use clap::Parser;
use clap::Subcommand;

#[derive(clap::Args)]
struct TraceArgs {
    #[clap(
        short,
        long,
        default_value = "trace.ndjson",
        help = "Output trace file path"
    )]
    output: String,

    #[clap(last = true, help = "Command and arguments to trace")]
    cmd_args: Vec<String>,
}

#[derive(clap::Args)]
struct ConvertArgs {
    #[clap(
        short,
        long,
        default_value = "trace.ndjson",
        help = "Input trace file path"
    )]
    input: String,

    #[clap(
        short,
        long,
        default_value = "chrome_trace.json",
        help = "Output Chrome trace file path"
    )]
    output: String,

    #[clap(long, help = "Generate flow events")]
    flow: bool,

    #[clap(
        long,
        default_missing_value = "0",
        num_args=0..=1,
        help = "Compact Timeline by reassigning thread IDs [default: number of hardware threads]"
    )]
    compact: Option<u32>,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Trace a command and record its execution time, \
        e.g., cc-timeline trace -- gcc -c file.c -o file.o")]
    Trace(TraceArgs),
    #[clap(about = "Convert a trace file to Chrome trace format")]
    Convert(ConvertArgs),
}

#[derive(Parser)]
#[clap(
    name = "cc-timeline",
    version,
    about = "A tool to trace execution times of compile commands \
    and visualize them in Chrome Trace format"
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

fn exec_with_measure(
    program: &str,
    args: &[&String],
) -> anyhow::Result<(u32, ExitStatus, SystemTime, Duration)> {
    let start = std::time::SystemTime::now();
    let mut child = std::process::Command::new(program)
        .args(args)
        .spawn()
        .expect("Failed to spawn command");
    let pid = child.id();
    let status = child.wait()?;
    let end = std::time::SystemTime::now();
    let duration = end.duration_since(start)?;
    Ok((pid, status, start, duration))
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

    let (pid, status, start, duration) =
        exec_with_measure(program, &program_args).expect("Failed to execute the command");
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    let event = TraceEvent::from(pid, start, duration, args.cmd_args.clone()).unwrap();
    let mut recorder =
        TraceRecorder::new(Path::new(&args.output)).expect("Failed to create trace recorder");
    recorder
        .record_event(&event)
        .expect("Failed to record trace event");
}

fn preprocess_trace_event(thread_placer: Option<&mut ThreadPlacer>, event: &mut TraceEvent) {
    match thread_placer {
        Some(tp) => {
            let new_tid = tp.place_thread(event.pid, event.timestamp_ns, event.duration_ns);
            event.pid = new_tid;
        }
        None => {}
    }
}

fn do_convert(args: &ConvertArgs) {
    let mut loader =
        TraceLoader::new(Path::new(&args.input)).expect("Failed to create trace loader");

    let mut thread_placer: Option<ThreadPlacer> = None;
    if let Some(compact_level) = args.compact {
        let level = if compact_level == 0 {
            // Get hardware concurrency
            std::thread::available_parallelism()
                .map(|n| n.get() as u32)
                .unwrap_or(64)
        } else {
            compact_level
        };
        thread_placer = Some(ThreadPlacer::new(level as usize));
    }

    let mut saver: ChromeTraceSaver =
        ChromeTraceSaver::new(Path::new(&args.output)).expect("Failed to create saver");
    if !args.flow {
        loader
            .load_events(|mut event: TraceEvent| {
                preprocess_trace_event(thread_placer.as_mut(), &mut event);
                saver.accept_trace_event(event)
            })
            .expect("Failed to load trace events");
    } else {
        let mut dep_manager = dependency::DependencyManager::new();
        loader
            .load_events(|mut event: TraceEvent| {
                preprocess_trace_event(thread_placer.as_mut(), &mut event);
                saver.accept_trace_event(event.clone());
                dep_manager.add_event(event);
            })
            .expect("Failed to load trace events");

        let mut flow_id: u64 = 0;
        dep_manager.iterate_dependencies(|parent, child| {
            let from_ns = parent.timestamp_ns + parent.duration_ns;
            let to_ns = child.timestamp_ns;
            saver.accept_flow_event(
                parent.name.clone(),
                from_ns,
                to_ns,
                parent.pid,
                child.pid,
                flow_id,
            );
            flow_id += 1;
        });
    }

    saver.save().expect("Failed to save Chrome trace file");
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Trace(trace_args) => do_trace(trace_args),
        Commands::Convert(convert_args) => do_convert(convert_args),
    }
}

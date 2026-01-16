use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Activate debug mode
    #[clap(short, long)]
    debug: bool,

    #[clap(last = true)]
    cmd_args: Vec<String>,
}

fn main() {
    let args = Cli::parse();

    if args.debug {
        println!("Debug mode is on");
    } else {
        println!("Debug mode is off");
    }

    if !args.cmd_args.is_empty() {
        println!("Additional arguments: {:?}", args.cmd_args);
    } else {
        println!("No additional arguments provided.");
    }
}

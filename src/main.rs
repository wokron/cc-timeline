use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Activate debug mode
    #[clap(short, long)]
    debug: bool,
}

fn main() {
    let args = Cli::parse();

    if args.debug {
        println!("Debug mode is on");
    } else {
        println!("Debug mode is off");
    }
}

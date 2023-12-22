#![feature(error_iter, try_blocks)]
mod engine;
use std::{error::Error, fs::File, io, path::PathBuf};

use clap::{Parser, Subcommand};
use engine::{Engine, MessageReader};
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Parser)]
#[command(version = "0.1.0")]
#[command(about = "CEGO-complaint experimental Chess engine")]
struct Cli {
    #[arg(
        short,
        long,
        help = "Activate tracing and write results to the specified file after truncating or creating it"
    )]
    trace_file: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Begin a CEGO session")]
    Run,
}

fn initialize_tracing(trace_file: PathBuf) -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_writer(File::create(trace_file)?)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::ACTIVE)
        .finish();

    Ok(tracing::subscriber::set_global_default(subscriber)?)
}

fn run() -> Result<(), Box<dyn Error>> {
    Engine::new(MessageReader::new(io::stdin().lock()))?.run()
}

pub fn cli() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    if let Some(trace_file) = cli.trace_file {
        initialize_tracing(trace_file)?;
    }

    match cli.command {
        Command::Run => run(),
    }
}

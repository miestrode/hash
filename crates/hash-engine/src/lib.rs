mod engine;
use std::{error::Error, fs::File, io, path::PathBuf};

use clap::{
    builder::{
        styling::{AnsiColor, Effects},
        Styles,
    },
    Parser, Subcommand,
};
use engine::{Engine, MessageReader};
use tracing::Level;

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::BrightGreen.on_default() | Effects::BOLD)
        .usage(AnsiColor::BrightGreen.on_default() | Effects::BOLD)
        .literal(AnsiColor::BrightBlue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default())
}

#[derive(Parser)]
#[command(styles = styles())]
#[command(version = "0.1.0")]
#[command(about = "CEGO-complaint experimental Chess engine")]
struct Cli {
    #[arg(
        short,
        long,
        help = "Activate tracing and write results to the specified file after truncating or creating it"
    )]
    trace_file: Option<PathBuf>,
    #[arg(
        requires("trace_file"),
        short = 'l',
        long,
        help = "Sets which events to filter from the trace. May be `trace`, `debug`, `info`, `warn` or `error`, in increasing order of restrictiveness. `trace` shows all events, `debug`, all events, not including `trace` events, etc. To use, `--trace-file` must be specified."
    )]
    tracing_level: Option<Level>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Begin a CEGO session")]
    Run,
}

fn initialize_tracing(trace_file: PathBuf, tracing_level: Level) -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_writer(File::create(trace_file)?)
        .event_format(tracing_subscriber::fmt::format().without_time().json())
        .with_thread_ids(true)
        .with_max_level(tracing_level)
        .finish();

    Ok(tracing::subscriber::set_global_default(subscriber)?)
}

fn run() -> Result<(), Box<dyn Error>> {
    Engine::new(MessageReader::new(io::stdin().lock()))?.run()
}

pub fn cli() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    if let Some(trace_file) = cli.trace_file {
        initialize_tracing(trace_file, cli.tracing_level.unwrap_or(Level::INFO))?;
    }

    match cli.command {
        Command::Run => run(),
    }
}

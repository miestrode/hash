mod engine;
use std::{error::Error, fs::File, io, path::PathBuf};

use clap::{
    builder::{styling::AnsiColor, Styles},
    Parser, Subcommand,
};
use engine::{Engine, EngineParameters, MessageReader};
use tracing::Level;

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default())
        .usage(AnsiColor::Yellow.on_default())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::BrightCyan.on_default())
}

#[derive(Parser)]
#[command(styles = styles())]
#[command(version = "0.1.0")]
#[command(about = "CEGO-complaint experimental Chess engine")]
struct Cli {
    #[arg(
        long,
        help = "Activate tracing and write results to the specified file after truncating or creating it"
    )]
    trace_file: Option<PathBuf>,
    #[arg(
        requires("trace_file"),
        short = 'l',
        long,
        help = "Sets which events to filter from the trace. May be `trace`, `debug`, `info`, `warn` or `error`, in increasing order of restrictiveness. `trace` shows all events, `debug`, all events, not including `trace` events, etc. `info` is the default value. To use, `--trace-file` must be specified.",
        default_value_t = Level::INFO
    )]
    tracing_level: Level,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Begin a CEGO session")]
    Run {
        #[arg(
            short = 't',
            long,
            help = "The number of search threads to use while searching.",
            default_value_t = 1
        )]
        search_threads: usize,
        #[arg(
            short = 'e',
            long,
            help = "The exploration rate to use for PUCT.",
            default_value_t = 4.0
        )]
        exploration_rate: f32,
    },
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

fn run(search_threads: usize, exploration_rate: f32) -> Result<(), Box<dyn Error>> {
    Engine::new(
        EngineParameters {
            search_threads,
            exploration_rate,
        },
        MessageReader::new(io::stdin().lock()),
    )?
    .run()
}

pub fn cli() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    if let Some(trace_file) = cli.trace_file {
        initialize_tracing(trace_file, cli.tracing_level)?;
    }

    match cli.command {
        Command::Run {
            search_threads,
            exploration_rate,
        } => run(search_threads, exploration_rate),
    }
}

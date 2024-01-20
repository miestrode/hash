use std::{
    self,
    borrow::BorrowMut,
    error::Error,
    fmt::Display,
    io::{BufRead, Lines, StdinLock},
    iter,
    num::ParseIntError,
    str::FromStr,
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration,
};

use burn_wgpu::Wgpu;
use mangrove_core::{
    board::{Board, ParseBoardError},
    repr::{ChessMove, ParseChessMoveError},
};
use mangrove_pisa::PisaConfig;
use mangrove_search::{
    search::{self, SearchCommand},
    tree::Tree,
};
use tracing::instrument;

#[derive(Debug)]
struct TimeData {
    time_left: Duration,
    opponent_time_left: Duration,
}

#[derive(Debug)]
struct IncrementData {
    increment: Duration,
    opponent_increment: Duration,
}

struct InitialMessage {
    times: TimeData,
    increments: IncrementData,
    board: Board,
}

#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum ParseInitialMessageError {
    #[error("initial message have 5 parts")]
    InvalidPartAmount,
    #[error("time must be an unsigned 64-bit integer")]
    InvalidTime(#[source] ParseIntError),
    #[error("position must be valid fen")]
    InvalidBoard(#[source] ParseBoardError),
}

impl FromStr for InitialMessage {
    type Err = ParseInitialMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(5, ' ');

        let mut times = parts
            .borrow_mut()
            .take(4)
            .map(|time| {
                time.parse::<u64>()
                    .map(Duration::from_nanos)
                    .map_err(ParseInitialMessageError::InvalidTime)
            })
            .chain(iter::repeat(Err(
                ParseInitialMessageError::InvalidPartAmount,
            )));

        Ok(Self {
            times: TimeData {
                time_left: times.next().unwrap()?,
                opponent_time_left: times.next().unwrap()?,
            },
            increments: IncrementData {
                increment: times.next().unwrap()?,
                opponent_increment: times.next().unwrap()?,
            },
            board: Board::from_str(
                parts
                    .next()
                    .ok_or(ParseInitialMessageError::InvalidPartAmount)?,
            )
            .map_err(ParseInitialMessageError::InvalidBoard)?,
        })
    }
}

struct SubsequentMessage {
    times: TimeData,
    played_move: ChessMove,
}

#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum ParseSubsequentMessageError {
    #[error("subsequent messsage must have 3 parts")]
    InvalidPartAmount,
    #[error("time must be an unsigned 64-bit integer")]
    InvalidTime(#[source] ParseIntError),
    #[error("move must be in uci notation")]
    InvalidMove(#[source] ParseChessMoveError),
}

impl FromStr for SubsequentMessage {
    type Err = ParseSubsequentMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(' ').map(Ok).chain(iter::repeat(Err(
            ParseSubsequentMessageError::InvalidPartAmount,
        )));

        Ok(Self {
            times: TimeData {
                time_left: Duration::from_nanos(
                    parts
                        .next()
                        .unwrap()?
                        .parse::<u64>()
                        .map_err(ParseSubsequentMessageError::InvalidTime)?,
                ),
                opponent_time_left: Duration::from_nanos(
                    parts
                        .next()
                        .unwrap()?
                        .parse::<u64>()
                        .map_err(ParseSubsequentMessageError::InvalidTime)?,
                ),
            },
            played_move: ChessMove::from_str(parts.next().unwrap()?)
                .map_err(ParseSubsequentMessageError::InvalidMove)?,
        })
    }
}

pub struct MessageReader<'a>(Lines<StdinLock<'a>>);

impl<'a> MessageReader<'a> {
    pub fn new(stdin_lock: StdinLock<'a>) -> Self {
        Self(stdin_lock.lines())
    }

    fn read_initial_message(&mut self) -> Result<InitialMessage, Box<dyn Error>> {
        Ok(
            InitialMessage::from_str(&self.0.next().ok_or(ProtocolError::InputStreamClosed)??)
                .map_err(ProtocolError::InvalidInitialMessage)?,
        )
    }

    fn read_subsequent_message(&mut self) -> Result<SubsequentMessage, Box<dyn Error>> {
        Ok(
            SubsequentMessage::from_str(&self.0.next().ok_or(ProtocolError::InputStreamClosed)??)
                .map_err(ProtocolError::InvalidSubsequentMessage)?,
        )
    }
}

pub struct Engine<'a> {
    command_sender: Sender<SearchCommand>,
    best_move_receiver: Receiver<ChessMove>,
    times: TimeData,
    increments: IncrementData,
    message_reader: MessageReader<'a>,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ProtocolError {
    #[error("invalid initial message")]
    InvalidInitialMessage(#[source] ParseInitialMessageError),
    #[error("invalid subsequent message")]
    InvalidSubsequentMessage(#[source] ParseSubsequentMessageError),
    #[error("input stream closed")]
    InputStreamClosed,
}

enum OutgoingMessage {
    Ready,
    BestMove(ChessMove),
}

impl Display for OutgoingMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ready => "ready\n".fmt(f),
            Self::BestMove(chess_move) => writeln!(f, "{chess_move}"),
        }
    }
}

pub struct EngineParameters {
    pub search_threads: usize,
    pub exploration_rate: f32,
}

impl<'a> Engine<'a> {
    #[instrument(name = "init engine", skip_all)]
    pub fn new(
        engine_parameters: EngineParameters,
        mut message_reader: MessageReader<'a>,
    ) -> Result<Self, Box<dyn Error>> {
        let network = PisaConfig::new().init::<Wgpu>();
        tracing::info!("initialized network");

        Self::send_message(OutgoingMessage::Ready);

        let InitialMessage {
            times,
            increments,
            board,
        } = message_reader.read_initial_message()?;

        tracing::info!(
            times = ?times,
            increments = ?increments,
            board = %board,
            "received initial message",
        );

        let (command_sender, best_move_receiver) = search::start_search_thread(
            Tree::new(board),
            network,
            engine_parameters.exploration_rate,
        );

        tracing::info!("started search thread");

        Ok(Self {
            command_sender,
            best_move_receiver,
            times,
            increments,
            message_reader,
        })
    }

    fn send_message(outgoing_message: OutgoingMessage) {
        print!("{outgoing_message}");
    }

    fn calculate_thinking_time(&self) -> Duration {
        Duration::from_secs(10)
    }

    fn think(&mut self) -> Result<(), Box<dyn Error>> {
        thread::sleep(self.calculate_thinking_time());

        self.command_sender
            .send(SearchCommand::SendAndPlayBestMove)?;
        let best_move = self.best_move_receiver.recv()?;

        Self::send_message(OutgoingMessage::BestMove(best_move));

        Ok(())
    }

    fn ponder(&mut self) -> Result<(), Box<dyn Error>> {
        let SubsequentMessage { times, played_move } =
            self.message_reader.read_subsequent_message()?;

        tracing::info!(
            times = ?times,
            played_move = %played_move,
            "received subsequent message",
        );

        self.times = times;
        self.command_sender
            .send(SearchCommand::PlayedMove(played_move))?;

        Ok(())
    }

    pub fn run(mut self) -> Result<(), Box<dyn Error>> {
        loop {
            tracing::info_span!("thinking").in_scope(|| self.think())?;
            tracing::info_span!("pondering").in_scope(|| self.ponder())?;
        }
    }
}

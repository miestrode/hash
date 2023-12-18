use std::{
    self,
    borrow::BorrowMut,
    error::Error,
    io::{BufRead, Lines, StdinLock},
    iter,
    num::ParseIntError,
    str::FromStr,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use hash_core::{
    board::{Board, ParseBoardError},
    repr::{ChessMove, ParseChessMoveError},
};
use hash_search::{
    search::{SearchCommand, SearchThread},
    tree::Tree,
};

struct TimeData {
    time_left: Duration,
    opponent_time_left: Duration,
}

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

// FIXME: This should collect to a vector and use a length check, anything else is invalid
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
    search_thread: SearchThread,
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

impl<'a> Engine<'a> {
    pub fn new(mut message_reader: MessageReader<'a>) -> Result<Self, Box<dyn Error>> {
        let (command_sender, command_receiver) = mpsc::channel();
        let (best_move_sender, best_move_receiver) = mpsc::channel();

        let InitialMessage {
            times,
            increments,
            board,
        } = message_reader.read_initial_message()?;

        Ok(Self {
            search_thread: SearchThread::new(Tree::new(board), command_receiver, best_move_sender),
            command_sender,
            best_move_receiver,
            times,
            increments,
            message_reader,
        })
    }

    fn calculate_thinking_time(&self) -> Duration {
        Duration::from_secs(5)
    }

    fn think(&mut self) -> Result<(), Box<dyn Error>> {
        thread::sleep(self.calculate_thinking_time());

        self.command_sender
            .send(SearchCommand::SendAndPlayBestMove)?;

        println!("{}", self.best_move_receiver.recv()?);

        Ok(())
    }

    fn ponder(&mut self) -> Result<(), Box<dyn Error>> {
        let SubsequentMessage { times, played_move } =
            self.message_reader.read_subsequent_message()?;

        self.times = times;
        self.command_sender
            .send(SearchCommand::PlayedMove(played_move))?;

        Ok(())
    }

    pub fn run(mut self) -> Result<(), Box<dyn Error>> {
        loop {
            self.think()?;
            self.ponder()?;
        }
    }
}

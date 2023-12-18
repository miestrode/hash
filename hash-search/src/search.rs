use crate::{puct::PuctSelector, tree::Tree};
use burn_wgpu::Wgpu;
use hash_core::repr::ChessMove;
use hash_network::model::ModelConfig;

use std::{
    any::Any,
    error::Error,
    sync::mpsc::{Receiver, Sender, TryRecvError},
    thread::{self, JoinHandle},
};

const EXPLORATION_RATE: f32 = 4.0;

pub enum SearchCommand {
    SendAndPlayBestMove,
    PlayedMove(ChessMove),
}

#[derive(Debug, thiserror::Error)]
pub enum SearchThreadError {
    #[error("search thread panicked: {0}")]
    Error(#[source] Box<dyn Error + 'static>),
    #[error("search thread panicked")]
    Unknown,
}

impl SearchThreadError {
    fn new(payload: Box<dyn Any + Send>) -> Self {
        match payload
            .downcast::<String>()
            .map(|string| string.as_str().into())
            .or_else(|payload| payload.downcast::<&str>().map(|string| (*string).into()))
            .ok()
        {
            Some(error) => Self::Error(error),
            None => Self::Unknown,
        }
    }
}

// NOTE: There is no `Drop` implementation, as the search thread is designed to stop when the
// channels are closed.
pub struct SearchThread(JoinHandle<()>);

impl SearchThread {
    pub fn new(
        mut tree: Tree,
        command_receiver: Receiver<SearchCommand>,
        best_move_sender: Sender<ChessMove>,
    ) -> Self {
        Self(thread::spawn(move || {
            let mut selector = PuctSelector::new(EXPLORATION_RATE);
            let network = ModelConfig::new().init::<Wgpu>();

            loop {
                match command_receiver.try_recv() {
                    Err(TryRecvError::Empty) => tree.expand(&mut selector, &network),
                    Ok(command) => match command {
                        SearchCommand::SendAndPlayBestMove => {
                            let best_move = tree.best_move();

                            if best_move_sender.send(best_move).is_err() {
                                return;
                            }

                            tree = tree.advance(best_move).unwrap();
                        }
                        SearchCommand::PlayedMove(chess_move) => {
                            tree = tree
                                .advance(chess_move)
                                .expect("opponent move is impossible")
                        }
                    },
                    Err(TryRecvError::Disconnected) => return,
                }
            }
        }))
    }

    pub fn join(self) -> Result<(), SearchThreadError> {
        self.0.join().map_err(SearchThreadError::new)
    }
}

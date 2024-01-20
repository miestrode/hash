use crate::tree::Tree;
use burn::tensor::backend::Backend;
use mangrove_core::repr::ChessMove;
use mangrove_pisa::Pisa;

use std::{
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
};

pub enum SearchCommand {
    SendAndPlayBestMove,
    PlayedMove(ChessMove),
}

pub fn start_search_thread<B: Backend>(
    mut tree: Tree,
    network: Pisa<B>,
    exploration_rate: f32,
) -> (Sender<SearchCommand>, Receiver<ChessMove>) {
    let (command_sender, command_receiver) = mpsc::channel();
    let (best_move_sender, best_move_receiver) = mpsc::channel();

    thread::spawn(move || loop {
        match command_receiver.try_recv() {
            Err(TryRecvError::Empty) => {
                tracing::trace!("growing tree");

                tree.grow(&network, exploration_rate);
            }
            Ok(command) => match command {
                SearchCommand::SendAndPlayBestMove => {
                    let best_move = tree.best_move().unwrap();

                    tracing::info!(%best_move, "found best move");

                    if best_move_sender.send(best_move).is_err() {
                        return;
                    }

                    tracing::info!(%best_move, "growing tree");

                    tree.try_advance(best_move).unwrap();
                }
                SearchCommand::PlayedMove(chess_move) => {
                    tracing::info!(%chess_move, "received opponent move");

                    tree.try_advance(chess_move)
                        .expect("opponent move is illegal or invalid");
                }
            },
            Err(TryRecvError::Disconnected) => return,
        }
    });

    (command_sender, best_move_receiver)
}

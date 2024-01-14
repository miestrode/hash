use crate::tree::Tree;
use burn::tensor::backend::Backend;
use hash_core::{mg, repr::ChessMove};
use hash_network::model::H0;

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
    network: H0<B>,
    exploration_rate: f32,
) -> (Sender<SearchCommand>, Receiver<ChessMove>) {
    let (command_sender, command_receiver) = mpsc::channel();
    let (best_move_sender, best_move_receiver) = mpsc::channel();

    thread::spawn(move || loop {
        match command_receiver.try_recv() {
            Err(TryRecvError::Empty) => {
                tracing::trace!("expanding tree");

                let (path, boards) = tree.select(exploration_rate, network.move_history());
                let end_board = boards.last().unwrap();
                let network_result = &network.process(vec![&boards])[0];

                tree.expand(
                    *path.last().unwrap(),
                    &mg::gen_moves(end_board)
                        .into_iter()
                        .map(|chess_move| {
                            (network_result.move_probabilities[chess_move], chess_move)
                        })
                        .collect::<Vec<_>>(),
                );

                // SAFETY: The path was obtained from `Tree::select`
                unsafe { tree.backpropagate(network_result.value, &path) };
            }
            Ok(command) => match command {
                SearchCommand::SendAndPlayBestMove => {
                    let best_move = tree.best_move().unwrap();

                    tracing::info!(%best_move, "found best move");

                    if best_move_sender.send(best_move).is_err() {
                        return;
                    }

                    tracing::info!(%best_move, "advancing tree");

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

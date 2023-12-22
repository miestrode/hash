use crate::{
    network::Network,
    tree::{Selector, Tree},
};
use hash_core::repr::ChessMove;

use std::{
    sync::mpsc::{Receiver, Sender, TryRecvError},
    thread,
};

pub enum SearchCommand {
    SendAndPlayBestMove,
    PlayedMove(ChessMove),
}

pub fn start_search_thread(
    mut tree: Tree,
    mut selector: impl Selector + Send + 'static,
    network: impl Network + Send + 'static,
    command_receiver: Receiver<SearchCommand>,
    best_move_sender: Sender<ChessMove>,
) {
    thread::spawn(move || loop {
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
    });
}

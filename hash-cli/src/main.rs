use std::{str::FromStr, thread};

use hash_core::game::Game;
use hash_eval::BasicEvaluator;

fn main() {
    let mut game =
        Game::from_str("5nk1/q2n2p1/2Q1p2P/1pPr4/3B3P/r4P2/8/1BRR2K1 b - - 2 37").unwrap();
    let chess_move = thread::Builder::new()
        .stack_size(2 * 1024 * 1024 * 1024)
        .spawn(move || hash_search::search(&mut game, &BasicEvaluator, 10).unwrap())
        .unwrap()
        .join()
        .unwrap();

    println!("{}", chess_move);
}

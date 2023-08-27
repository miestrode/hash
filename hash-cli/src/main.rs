use std::{str::FromStr, thread};

use hash_core::game::Game;
use hash_eval::BasicEvaluator;

fn main() {
    let mut game =
        Game::from_str("6k1/3b3r/1p1p4/p1n2p2/1PPNpP1q/P3Q1p1/1R1RB1P1/5K2 b - - 0 1").unwrap();
    let chess_move = thread::Builder::new()
        .stack_size(2 * 1024 * 1024 * 1024)
        .spawn(move || hash_search::search(&mut game, &BasicEvaluator, 6, 1.0).unwrap())
        .unwrap()
        .join()
        .unwrap();

    println!("{}", chess_move);
}

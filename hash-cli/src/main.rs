use std::{str::FromStr, thread};

use hash_core::game::Game;
use hash_eval::BasicEvaluator;

fn main() {
    let mut game =
        Game::from_str("r2qrk2/1ppbb3/2n3pp/p3R3/P2PR3/BQP2N2/5PPP/6K1 w - - 0 1").unwrap();
    let chess_move = thread::Builder::new()
        .stack_size(2 * 1024 * 1024 * 1024)
        .spawn(move || hash_search::search(&mut game, &BasicEvaluator, 19, 1.0).unwrap())
        .unwrap()
        .join()
        .unwrap();

    println!("{}", chess_move);
}

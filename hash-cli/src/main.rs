use std::thread;

use hash_core::game::Game;
use hash_eval::BasicEvaluator;

fn main() {
    let game = Game::default();
    let (chess_move, evaluation) = thread::Builder::new()
        .stack_size(2 * 1024 * 1024 * 1024)
        .spawn(move || hash_search::search(&game, &BasicEvaluator, 6).unwrap())
        .unwrap()
        .join()
        .unwrap();

    println!("{}: {}", chess_move, evaluation);
}

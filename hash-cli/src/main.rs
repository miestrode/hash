use std::str::FromStr;

use hash_core::game::Game;
use hash_eval::BasicEvaluator;

fn main() {
    let game = Game::from_str("1r2r1k1/p1pbqppp/Q2b1n2/3p4/P2P4/2P5/1P2BPPP/R1B1KN1R b KQ - 2 14")
        .unwrap();
    let (chess_move, evaluation) = hash_search::search(&game, &BasicEvaluator, 4).unwrap();

    println!("{}, {}", chess_move, evaluation);
}

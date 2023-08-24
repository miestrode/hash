use hash_core::game::Game;
use hash_eval::BasicEvaluator;

fn main() {
    let game = Game::default();
    let (chess_move, evaluation) = hash_search::search(&game, &BasicEvaluator, 6).unwrap();

    println!("{}: {}", chess_move, evaluation);
}

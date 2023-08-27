use hash_core::{board::Board, game::Game, repr::Move};
use score::Score;

pub mod score;
mod search;
mod tt;

pub trait Eval {
    fn eval(&self, board: &Board) -> Score;
}

pub fn search<E: Eval + Sync>(
    game: &mut Game,
    evaluator: &E,
    max_depth: i16,
    quality: f32,
) -> Option<Move> {
    if game.outcome().is_none() {
        let table = tt::ConcreteTable::new();

        search::optimize(game, evaluator, &table, max_depth, quality)
    } else {
        None
    }
}

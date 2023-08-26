use hash_core::{board::Board, game::Game, repr::Move};
use score::Score;

pub mod score;
mod search;
mod tt;

pub trait Eval {
    fn eval(&self, board: &Board) -> Score;
}

// TODO: Make search interruptible
pub fn search<E: Eval + Sync>(game: &Game, evaluator: &E, depth: i16) -> Option<(Move, Score)> {
    let table = tt::ConcreteTable::new();

    if game.outcome().is_none() {
        game.following_games()
            .into_iter()
            .map(|(chess_move, game)| {
                (
                    chess_move,
                    search::negamax(
                        &game,
                        evaluator,
                        &table,
                        depth - 1,
                        depth,
                        Score::WORST,
                        Score::BEST,
                    )
                    .flip(),
                )
            })
            .max_by_key(|(_, evaluation)| *evaluation)
    } else {
        None
    }
}

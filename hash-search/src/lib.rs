use hash_core::{board::Board, game::Game, repr::Move};
use score::Score;

pub mod score;
mod search;
mod tt;

pub trait Eval {
    fn eval(&self, board: &Board) -> Score;
}

// TODO: Make search interruptible
/*
pub fn search<E: Eval + Sync>(game: &mut Game, evaluator: &E, depth: i16) -> Option<(Move, Score)> {

    if game.outcome().is_none() {
        mg::gen_moves(&game.board)
            .into_iter()
            .map(|chess_move| {
                unsafe { game.make_move_unchecked(&chess_move) };
                let result = (
                    chess_move,
                    search::negamax(
                        game,
                        evaluator,
                        &table,
                        depth - 1,
                        depth,
                        Score::WORST,
                        Score::BEST,
                    )
                    .flip(),
                );
                game.unmake_last_move();

                result
            })
            .max_by_key(|(_, evaluation)| *evaluation)
    } else {
        None
    }
}
*/

pub fn search<E: Eval + Sync>(
    game: &mut Game,
    evaluator: &E,
    repetitions_to_end: usize,
) -> Option<Move> {
    if game.outcome().is_none() {
        let table = tt::ConcreteTable::new();
        search::optimize(game, evaluator, &table, repetitions_to_end)
    } else {
        None
    }
}

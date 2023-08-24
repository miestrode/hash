use std::thread;

use hash_core::{board::Board, game::Game, repr::Move};
use score::Score;

pub mod score;
mod search;
mod tt;

pub trait Eval {
    fn eval(&self, board: &Board) -> Score;
}

pub const BYTES_IN_GIB: usize = 1024 * 1024 * 1024;
pub const STACK_SIZE_IN_BYTES: usize = 8 * BYTES_IN_GIB;

// TODO: Make search interruptible
pub fn search<E: Eval + Sync>(game: &Game, evaluator: &E, depth: i16) -> Option<(Move, Score)> {
    thread::scope(|s| {
        thread::Builder::new()
            .stack_size(STACK_SIZE_IN_BYTES)
            .spawn_scoped(s, || {
                let mut table = tt::Table::new();

                if game.outcome().is_none() {
                    game.following_games()
                        .into_iter()
                        .map(|(chess_move, game)| {
                            (
                                chess_move,
                                search::negamax(
                                    &game,
                                    evaluator,
                                    &mut table,
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
            })
            .unwrap()
            .join()
            .unwrap()
    })
}

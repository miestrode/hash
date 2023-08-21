use hash_core::{
    board::Board,
    game::{Game, Outcome},
    repr::Move,
    Color,
};
use score::Score;

pub mod score;

pub trait Eval {
    fn eval(&self, board: &Board) -> Score;
}

fn negamax<E: Eval + Sync>(
    game: &Game,
    evaluator: &E,
    depth: u32,
    steps: i16,
    mut alpha: Score,
    beta: Score,
    calls: &mut u64,
) -> Score {
    if let Some(outcome) = game.outcome() {
        match outcome {
            Outcome::Draw => Score::DRAW,
            _ => Score::from_mate_distance(-steps), // A mate cannot be of the current player
        }
    } else if depth == 0 {
        let mut evaluation = evaluator.eval(&game.board);

        if game.board.current_color == Color::Black {
            evaluation.flip_in_place();
        }

        *calls += 1;

        evaluation
    } else {
        let mut score = Score::WORST;

        for (_, game) in game.following_games() {
            score = score.max(
                negamax(
                    &game,
                    evaluator,
                    depth - 1,
                    steps + 1,
                    beta.flip(),
                    alpha.flip(),
                    calls,
                )
                .flip(),
            );
            alpha = alpha.max(score);

            if score >= beta {
                break;
            }
        }

        score
    }
}

// TODO: Make search interruptible
pub fn search<E: Eval + Sync>(game: &Game, evaluator: &E, depth: u32) -> Option<(Move, Score)> {
    let mut calls = 0;

    if game.outcome().is_none() {
        let result = game
            .following_games()
            .into_iter()
            .map(|(chess_move, game)| {
                (
                    chess_move,
                    negamax(
                        &game,
                        evaluator,
                        depth - 1,
                        1,
                        Score::WORST,
                        Score::BEST,
                        &mut calls,
                    )
                    .flip(),
                )
            })
            .reduce(|a, b| if a.1 < b.1 { b } else { a });
        println!("{}", calls);
        result
    } else {
        None
    }
}

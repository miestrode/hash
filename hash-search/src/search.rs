use std::{convert, time::Instant};

use hash_core::{
    game::{Game, Outcome},
    mg::{self, Moves},
    repr::Move,
};

use crate::{
    score::Score,
    tt::{self, Entry, EntryMetadata},
    Eval,
};

fn negamax<E: Eval + Sync, const N: usize>(
    game: &mut Game,
    evaluator: &E,
    tt: &tt::Table<N>,
    depth: i16,
    start_depth: i16,
    mut alpha: Score,
    mut beta: Score,
) -> Score {
    let original_alpha = alpha;

    // TODO: Change this once we have let-expressions in argument position
    if let Some(Entry {
        evaluation,
        metadata,
        depth: entry_depth,
        ..
    }) = tt.get(&game.board)
    {
        // TODO: Consider somehow utilizing the reselt of the shallower search, instead of just
        // discarding it.
        if entry_depth >= depth {
            match metadata {
                EntryMetadata::Exact => return evaluation,
                EntryMetadata::LowerBound => alpha = alpha.max(evaluation),
                EntryMetadata::UpperBound => beta = beta.min(evaluation),
            }

            if alpha >= beta {
                return evaluation;
            }
        }
    }

    if let Some(outcome) = game.outcome() {
        match outcome {
            Outcome::Draw => Score::DRAW,
            _ => Score::from_mate_distance(depth - start_depth), // A mate cannot be of the current player
        }
    } else if depth == 0 {
        evaluator.eval(&game.board)
    } else {
        let evaluation = mg::gen_moves(&game.board)
            .into_iter()
            .try_fold(Score::WORST, |mut best, chess_move| {
                unsafe { game.make_move_unchecked(&chess_move) };
                best = best.max(
                    negamax(
                        game,
                        evaluator,
                        tt,
                        depth - 1,
                        start_depth,
                        beta.flip(),
                        alpha.flip(),
                    )
                    .flip(),
                );
                game.unmake_last_move();

                alpha = alpha.max(best);

                if alpha >= beta {
                    Err(best)
                } else {
                    Ok(best)
                }
            })
            .unwrap_or_else(convert::identity);

        tt.insert(
            &game.board,
            evaluation,
            depth,
            if evaluation <= original_alpha {
                EntryMetadata::UpperBound
            } else if evaluation >= beta {
                EntryMetadata::LowerBound
            } else {
                EntryMetadata::Exact
            },
        );

        evaluation
    }
}

fn null_window_search<E: Eval + Sync, const N: usize>(
    game: &mut Game,
    evaluator: &E,
    tt: &tt::Table<N>,
    depth: i16,
    chess_move: &Move,
    guess: Score,
) -> Score {
    unsafe { game.make_move_unchecked(chess_move) };

    let result = negamax(
        game,
        evaluator,
        tt,
        depth - 1,
        depth,
        guess.flip(),
        Score::from_int(-(guess.as_int() - 1)),
    )
    .flip();

    game.unmake_last_move();

    result
}

// See: https://en.wikipedia.org/wiki/Best_node_search
fn bns<E: Eval + Sync, const N: usize>(
    game: &mut Game,
    evaluator: &E,
    tt: &tt::Table<N>,
    depth: i16,
    initial_guess: Score,
    quality: f32,
) -> (Move, Score) {
    let mut alpha = Score::WORST;
    let mut beta = Score::BEST;

    fn next_guess(alpha: Score, beta: Score, subtree_count: usize) -> Score {
        let subtree_count = subtree_count as i64;
        let alpha = alpha.as_int() as i64;
        let beta = beta.as_int() as i64;

        // There are a total of subtree_count subtrees above the guess we had. We only want one
        // subtree to remain. Therefore, we should have the new guess go up to subtree_count - 1 of
        // the way, assuming the subtree cutoff points are distributed uniformly.
        Score::from_int((alpha + (beta - alpha) * (subtree_count - 1) / subtree_count) as i16)
    }

    let mut candidates = mg::gen_moves(&game.board);
    let mut guess = initial_guess;
    let mut bound = 1;

    loop {
        let mut new_candidates = Moves::new();

        for (candidate, checked) in candidates.iter().zip(1..) {
            if null_window_search(game, evaluator, tt, depth, candidate, guess) >= guess {
                let expected_quality = 1.0
                    / ((new_candidates.len() + 1) as f32 / checked as f32
                        * candidates.len() as f32);

                if expected_quality >= quality {
                    return (*candidate, guess);
                }

                new_candidates.push(*candidate);
            }
        }

        println!("{alpha} <= {guess} <= {beta}: {}", new_candidates.len(),);

        if new_candidates.is_empty() {
            beta = guess;
        } else {
            candidates = new_candidates;
            alpha = guess;
        }

        if beta.as_int() <= alpha.as_int() + 1 || candidates.len() == 1 {
            break (candidates[0], guess);
        }

        guess = Score::from_int(
            guess.as_int()
                + (next_guess(alpha, beta, candidates.len()).as_int() - guess.as_int())
                    .clamp(-bound, bound),
        );
        bound *= 2;
    }
}

// TODO: Change the target quality during the deepening
// TODO: Add the ability to terminate early when it is believed that will be unharmful
pub(crate) fn optimize<E: Eval + Sync, const N: usize>(
    game: &mut Game,
    evaluator: &E,
    tt: &tt::Table<N>,
    max_depth: i16,
    quality: f32,
) -> Option<Move> {
    let mut best_move = None;
    let mut guess = Score::DRAW;
    let initial = Instant::now();

    for depth in 1..=max_depth {
        println!("START DEPTH {depth}");
        let time = Instant::now();
        let (new_best_move, new_guess) = bns(game, evaluator, tt, depth, guess, quality);
        println!("FINISH IN {}ms", time.elapsed().as_millis());

        best_move = Some(new_best_move);
        guess = new_guess;
    }

    println!("TOTAL TIME: {}ms", initial.elapsed().as_millis());

    best_move
}

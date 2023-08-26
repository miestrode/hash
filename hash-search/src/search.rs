use std::convert;

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

const CANDIDATE_HISTORY_START_CAPACITY: usize = 10;

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

fn null_window_test<E: Eval + Sync, const N: usize>(
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
        depth,
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
    mut alpha: Score,
    mut beta: Score,
) -> Moves {
    fn next_guess(mut alpha: i16, mut beta: i16, subtree_count: usize) -> Score {
        if alpha <= 0 {
            beta = beta.min(i16::MAX / 2);
        }

        if beta >= 0 {
            alpha = alpha.max(-i16::MAX / 2)
        }

        let mut guess = alpha
            + ((beta - alpha) as f32 * (subtree_count as f32 - 1.0) / subtree_count as f32) as i16;

        if guess == alpha {
            guess += 1;
        } else if guess == beta {
            guess -= 1;
        }

        Score::from_int(guess)
    }

    let mut candidates = mg::gen_moves(&game.board);

    loop {
        let guess = next_guess(alpha.as_int(), beta.as_int(), candidates.len());
        let mut new_candidates = Moves::new();

        for candidate in &candidates {
            if null_window_test(game, evaluator, tt, depth, candidate, guess) >= guess {
                new_candidates.push(*candidate);
            }
        }

        if new_candidates.is_empty() {
            beta = guess;
        } else {
            candidates = new_candidates;
            alpha = guess;
        }

        if beta.as_int() <= alpha.as_int() + 1 || candidates.len() == 1 {
            break candidates;
        }
    }
}

pub(crate) fn optimize<E: Eval + Sync, const N: usize>(
    game: &mut Game,
    evaluator: &E,
    tt: &tt::Table<N>,
    repetitions_necessary: usize,
) -> Option<Move> {
    let mut depth = 1;
    let mut candidates_history: Vec<Moves> = Vec::with_capacity(CANDIDATE_HISTORY_START_CAPACITY);

    fn get_matches(candidate: &Move, candidates_history: &[Moves]) -> usize {
        candidates_history
            .iter()
            .filter(|candidates| candidates.contains(candidate))
            .count()
    }

    loop {
        let candidates = bns(game, evaluator, tt, depth, Score::WORST, Score::BEST);

        for candidate in &candidates {
            print!("{candidate}, ");
        }
        println!();

        if !candidates.is_empty()
            && candidates.iter().all(|candidate| {
                get_matches(candidate, &candidates_history) >= repetitions_necessary
            })
        {
            break candidates
                .into_iter()
                .max_by_key(|candidate| get_matches(candidate, &candidates_history));
        }

        candidates_history.push(candidates);
        depth += 1;
    }
}

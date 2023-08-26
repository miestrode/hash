use std::convert;

use hash_core::game::{Game, Outcome};

use crate::{
    score::Score,
    tt::{self, Entry, EntryMetadata},
    Eval,
};

pub(crate) fn negamax<E: Eval + Sync, const N: usize>(
    game: &Game,
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
        match metadata {
            EntryMetadata::Exact => return evaluation,
            EntryMetadata::LowerBound => alpha = alpha.max(evaluation),
            EntryMetadata::UpperBound => beta = beta.min(evaluation),
        }

        if alpha >= beta {
            return evaluation;
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
        let evaluation = game
            .following_games()
            .into_iter()
            .try_fold(Score::WORST, |mut best, (_, game)| {
                best = best.max(
                    negamax(
                        &game,
                        evaluator,
                        tt,
                        depth - 1,
                        start_depth,
                        beta.flip(),
                        alpha.flip(),
                    )
                    .flip(),
                );
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

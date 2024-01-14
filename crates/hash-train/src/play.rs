// TODO: Refactor this whole file
use std::iter;

use burn::tensor::{backend::Backend, Tensor};
use hash_core::{
    board::Board,
    game::{Game, Outcome},
};
use hash_network::model::{H0Result, MoveProbabilities, H0};
use hash_search::{
    puct::PuctSelector,
    tree::{Selector, Tree},
};
use rand::{distributions::WeightedIndex, Rng};
use ringbuffer::{AllocRingBuffer, RingBuffer};

const EXPANSIONS: usize = 20;

fn expand_tree(
    tree: &mut Tree,
    selector: &mut impl Selector,
    model: &H0<impl Backend>,
    expansions: usize,
) {
    for _ in 0..expansions {
        tree.expand(selector, model);
    }
}

#[derive(Clone)]
pub struct TrainInput<B: Backend> {
    pub input: Tensor<B, 3>,
    pub expected_output: Tensor<B, 1>,
}

// TODO: Optimize and refactor this code and consider using const-generics for the move history as
// this could considerably improve performance here. Maybe using a global board array would also
// improve performance
pub fn gen_game<B: Backend>(
    model: &H0<B>,
    ply_cap: usize,
    rng: &mut impl Rng,
) -> Vec<TrainInput<B>> {
    let mut puct_selector = PuctSelector::new(4.0);
    let mut tree = Tree::new(Board::starting_position());

    let mut positions = Vec::with_capacity(ply_cap);
    let mut boards = AllocRingBuffer::new(model.move_history());
    let mut game = Game::starting_position();

    let outcome = loop {
        boards.push(*game.board());

        expand_tree(&mut tree, &mut puct_selector, model, EXPANSIONS);

        let tree_visits = tree.visits() as f32;
        let children = tree.children().unwrap();

        let move_probabilities = MoveProbabilities::new(
            children
                .iter()
                .map(|child| (child.tree.visits() as f32 / tree_visits, child.chess_move)),
        );

        let child_index = rng
            .sample(WeightedIndex::new(children.iter().map(|child| child.tree.visits())).unwrap());

        let child = children.into_iter().nth(child_index).unwrap();

        game.make_move(child.chess_move);

        tree = child.tree;

        positions.push((boards.to_vec(), move_probabilities));

        if positions.len() >= ply_cap {
            break Outcome::Draw;
        } else if let Some(outcome) = game.outcome() {
            break outcome;
        }
    };

    let finishing_color = game.board().playing_color;

    let outcome_value = match outcome {
        Outcome::Win(_) => 1.0,
        Outcome::Draw => 0.0,
    };

    // TODO: Consider splitting on the outcome in this section, or maybe splitting the boards into
    // ones of the color white and the color black
    positions
        .into_iter()
        .map(|(boards, move_probabilities)| TrainInput {
            input: hash_network::boards_to_tensor(
                boards
                    .iter()
                    .map(Some)
                    .chain(iter::repeat(None).take(model.move_history() - boards.len()))
                    .collect(),
            ),
            expected_output: H0Result {
                // If the finshing color, in other words, the color that would play if the game
                // wasn't finished, actually won, it would mean it would have the ability to
                // capture the opponent's king, and thus this color always loses.
                value: if finishing_color != boards.last().unwrap().playing_color {
                    1.0
                } else {
                    -1.0
                } * outcome_value,
                move_probabilities,
            }
            .into(),
        })
        .collect::<Vec<_>>()
}

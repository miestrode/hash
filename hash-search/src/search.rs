use crate::{puct::PuctSelector, tree::Tree};
use burn_ndarray::NdArrayBackend;
use hash_core::{board::Board, repr::Move};
use hash_network::model::ModelConfig;

const EXPANSIONS: usize = 100;
const EXPLORATION_RATE: f32 = 4.0;

pub fn search(board: Board) -> Move {
    let mut selector = PuctSelector::new(EXPLORATION_RATE);
    let network = ModelConfig::new().init::<NdArrayBackend<f32>>();

    let mut tree = Tree::new(board);

    for expansion in 0..EXPANSIONS {
        tree.expand(&mut selector, &network);
        println!("FINISHED EXPANSION {expansion}");
    }

    tree.best_move()
}

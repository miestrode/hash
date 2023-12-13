use crate::{puct::PuctSelector, tree::Tree};
use burn_wgpu::Wgpu;
use hash_network::model::ModelConfig;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

const EXPLORATION_RATE: f32 = 4.0;

/// TODO: Add configuration options to here, such as the level of exploration that is desired
/// (exploration rate). This will be useful for, for example, performing a more explorative search
/// when pondering, which should account for uncertainty.
pub fn search(mut tree: Tree, stop_search: Arc<AtomicBool>) -> Tree {
    let mut selector = PuctSelector::new(EXPLORATION_RATE);
    let network = ModelConfig::new().init::<Wgpu>();

    while !stop_search.load(Ordering::Relaxed) {
        tree.expand(&mut selector, &network);
    }

    tree
}

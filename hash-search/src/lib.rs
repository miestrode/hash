#![feature(cell_update)]

use crate::tree::Tree;
use hash_bootstrap::Square;
use hash_core::{
    board::Board,
    repr::{Move, PieceKind},
};

mod network;
mod puct;
pub mod search;
mod tree;

trait Selector {
    fn choose_child<'a>(&mut self, tree: &'a Tree) -> Option<&'a Tree>;
}

pub struct MoveProbabilities {
    probabilities: [f32; MoveProbabilities::ARRAY_LENGTH],
}

impl MoveProbabilities {
    const REGULAR_MOVE_SECTION_LENGTH: usize = 64 * 64;

    const SINGLE_PIECE_PROMOTION_SECTION_LENGTH: usize = 8 * 8;
    const SINGLE_RANK_PROMOTION_SECTION_LENGTH: usize =
        Self::SINGLE_PIECE_PROMOTION_SECTION_LENGTH * 4;

    const ARRAY_LENGTH: usize =
        Self::REGULAR_MOVE_SECTION_LENGTH + 2 * Self::SINGLE_RANK_PROMOTION_SECTION_LENGTH;

    // This function defines a one-to-one mapping between numbers from 0 to 4609 (non-inclusive) to
    // Chess moves, and back.
    fn probability(&self, chess_move: Move) -> f32 {
        if let Some(piece_kind) = chess_move.promotion {
            let promotion_number: usize = match piece_kind {
                PieceKind::Queen => 0,
                PieceKind::Rook => 1,
                PieceKind::Bishop => 2,
                PieceKind::Knight => 3,
                _ => unreachable!(),
            };

            let is_eighth_rank_promotion = chess_move.target.rank() == Square::RANK_8;

            self.probabilities[Self::REGULAR_MOVE_SECTION_LENGTH
                + chess_move.origin.file() as usize
                + 8 * chess_move.target.file() as usize
                + Self::SINGLE_PIECE_PROMOTION_SECTION_LENGTH * promotion_number
                + Self::SINGLE_RANK_PROMOTION_SECTION_LENGTH * (is_eighth_rank_promotion as usize)]
        } else {
            self.probabilities[chess_move.origin.as_index() + chess_move.target.as_index() * 64]
        }
    }
}

pub struct NetworkResult {
    pub value: f32,
    pub move_probabilities: MoveProbabilities,
}

pub trait Network {
    fn maximum_boards_expected(&self) -> usize;

    fn run(&self, boards: Vec<Board>) -> NetworkResult;
}

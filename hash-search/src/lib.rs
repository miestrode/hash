#![feature(cell_update, generic_const_exprs)]

use crate::tree::Tree;
use arrayvec::ArrayVec;
use hash_bootstrap::Square;
use hash_core::{
    board::Board,
    repr::{Move, PieceKind},
};
use rand::Rng;

mod tree;

trait Selector {
    fn choose_child<'a>(&mut self, tree: &'a Tree) -> Option<&'a Tree>;
}

pub struct SimplePolicy;

impl Selector for SimplePolicy {
    fn choose_child<'a>(&mut self, tree: &'a Tree) -> Option<&'a Tree> {
        // SAFETY: We don't mutate anything.
        unsafe { tree.children.as_ptr().as_ref() }
            .unwrap()
            .as_ref()
            .map(|children| {
                children
                    .iter()
                    .max_by(|child_a, child_b| child_a.probability.total_cmp(&child_b.probability))
                    .unwrap()
                    .tree
                    .as_ref()
            })
    }
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
    const MOVE_HISTORY: usize;

    fn run(&self, boards: ArrayVec<Board, { Self::MOVE_HISTORY }>) -> NetworkResult;
}

pub struct SimpleNetwork;

impl Network for SimpleNetwork {
    const MOVE_HISTORY: usize = 1;

    fn run(&self, _boards: ArrayVec<Board, { Self::MOVE_HISTORY }>) -> NetworkResult {
        NetworkResult {
            value: rand::thread_rng().gen_range(-1.0..1.0),
            move_probabilities: MoveProbabilities {
                probabilities: rand::random(),
            },
        }
    }
}

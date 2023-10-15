use burn::tensor::{backend::Backend, Tensor};
use hash_bootstrap::Square;
use hash_core::{
    board::Board,
    repr::{Move, PieceKind},
};
use hash_network::model::{BatchOutput, Model};
use num_traits::ToPrimitive;
use std::iter;

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

    pub fn new(probability_iter: impl Iterator<Item = (f32, Move)>) -> Self {
        let mut move_probabilities = Self {
            probabilities: [0.0; Self::ARRAY_LENGTH],
        };

        for (probability, chess_move) in probability_iter {
            move_probabilities.set_probability(chess_move, probability);
        }

        move_probabilities
    }

    fn move_to_index(chess_move: Move) -> usize {
        if let Some(piece_kind) = chess_move.promotion {
            let promotion_number: usize = match piece_kind {
                PieceKind::Queen => 0,
                PieceKind::Rook => 1,
                PieceKind::Bishop => 2,
                PieceKind::Knight => 3,
                _ => unreachable!(),
            };

            let is_eighth_rank_promotion = chess_move.target.rank() == Square::RANK_8;

            Self::REGULAR_MOVE_SECTION_LENGTH
                + chess_move.origin.file() as usize
                + 8 * chess_move.target.file() as usize
                + Self::SINGLE_PIECE_PROMOTION_SECTION_LENGTH * promotion_number
                + Self::SINGLE_RANK_PROMOTION_SECTION_LENGTH * (is_eighth_rank_promotion as usize)
        } else {
            chess_move.origin.as_index() + chess_move.target.as_index() * 64
        }
    }

    // This function defines a one-to-one mapping between numbers from 0 to 4609 (non-inclusive) to
    // Chess moves, and back.
    pub fn get_probability(&self, chess_move: Move) -> f32 {
        self.probabilities[Self::move_to_index(chess_move)]
    }

    pub fn set_probability(&mut self, chess_move: Move, probability: f32) {
        self.probabilities[Self::move_to_index(chess_move)] = probability;
    }
}

pub struct NetworkResult {
    pub value: f32,
    pub move_probabilities: MoveProbabilities,
}

impl<B: Backend> From<NetworkResult> for Tensor<B, 1> {
    fn from(value: NetworkResult) -> Self {
        Tensor::cat(
            vec![
                Tensor::from_floats(value.move_probabilities.probabilities),
                Tensor::from_floats([value.value]),
            ],
            0,
        )
    }
}

pub trait Network {
    fn maximum_boards_expected(&self) -> usize;

    fn run(&self, boards: Vec<Board>) -> NetworkResult;
}

impl<B: Backend> Network for Model<B> {
    fn maximum_boards_expected(&self) -> usize {
        self.move_history()
    }

    fn run(&self, boards: Vec<Board>) -> NetworkResult {
        let empty_boards = self.maximum_boards_expected() - boards.len();

        let boards = boards
            .iter()
            .map(Some)
            .chain(iter::repeat(None).take(empty_boards))
            .collect();

        let BatchOutput {
            values,
            probabilities,
        } = self.forward(hash_network::boards_to_tensor(boards).unsqueeze());

        NetworkResult {
            value: values.into_scalar().to_f32().unwrap(),
            move_probabilities: MoveProbabilities {
                probabilities: probabilities
                    .squeeze::<1>(0)
                    .into_data()
                    .convert::<f32>()
                    .value
                    .try_into()
                    .unwrap(),
            },
        }
    }
}

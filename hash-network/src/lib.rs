// TODO: Refactor this whole module, the code here is quite ugly.
use burn::tensor::{backend::Backend, Shape, Tensor};
use hash_bootstrap::{BitBoard, Color, Square};
use hash_core::{board::Board, repr::Player};

pub mod model;

fn stack<B: Backend, const D: usize, const D2: usize>(tensors: Vec<Tensor<B, D>>) -> Tensor<B, D2> {
    Tensor::cat(
        tensors
            .into_iter()
            .map(|tensor| tensor.unsqueeze())
            .collect(),
        0,
    )
}

fn bitboard_to_tensor<B: Backend>(bitboard: BitBoard) -> Tensor<B, 2> {
    Tensor::from_floats((Square::ALL).map(|square| f32::from(bitboard.get_bit(square))))
        .reshape(Shape::new([8, 8]))
}

fn player_to_tensor<B: Backend>(player: &Player) -> Tensor<B, 3> {
    stack(vec![
        bitboard_to_tensor(player.pawns),
        bitboard_to_tensor(player.knights),
        bitboard_to_tensor(player.bishops),
        bitboard_to_tensor(player.rooks),
        bitboard_to_tensor(player.queens),
        bitboard_to_tensor(player.king),
    ])
}

fn boolean_to_tensor<B: Backend>(boolean: bool) -> Tensor<B, 2> {
    if boolean {
        Tensor::ones(Shape::new([8, 8]))
    } else {
        Tensor::zeros(Shape::new([8, 8]))
    }
}

pub fn board_to_tensor<B: Backend>(board: Option<&Board>) -> Tensor<B, 3> {
    if let Some(board) = board {
        Tensor::cat(
            vec![
                player_to_tensor(&board.us),
                player_to_tensor(&board.them),
                bitboard_to_tensor(BitBoard::from(board.en_passant_capture_square)).unsqueeze(),
                boolean_to_tensor(board.us.castling_rights.can_castle_king_side()).unsqueeze(),
                boolean_to_tensor(board.us.castling_rights.can_castle_queen_side()).unsqueeze(),
                boolean_to_tensor(board.them.castling_rights.can_castle_king_side()).unsqueeze(),
                boolean_to_tensor(board.them.castling_rights.can_castle_queen_side()).unsqueeze(),
                match board.playing_color {
                    Color::White => Tensor::ones(Shape::new([8, 8])),
                    Color::Black => Tensor::ones(Shape::new([8, 8])).neg(),
                }
                .unsqueeze(),
                Tensor::from_floats([board.min_half_move_clock as f32; 64])
                    .reshape(Shape::new([1, 8, 8])),
                boolean_to_tensor(true).unsqueeze(), // This is for the existence layer
            ],
            0,
        )
    } else {
        Tensor::zeros(Shape::new([model::SINGLE_BOARD_DIMENSION, 8, 8]))
    }
}

pub fn boards_to_tensor<B: Backend>(boards: Vec<Option<&Board>>) -> Tensor<B, 3> {
    stack(
        boards
            .iter()
            .copied()
            .map(|board| board_to_tensor(board))
            .collect(),
    )
}

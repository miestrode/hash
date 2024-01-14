#![feature(array_chunks)]

use burn::tensor::{backend::Backend, Shape, Tensor};
use hash_bootstrap::{BitBoard, Color, Square};
use hash_core::{board::Board, repr::Player};

pub mod model;

fn bitboard_to_tensor<B: Backend>(bitboard: BitBoard) -> Tensor<B, 2> {
    Tensor::from_floats((Square::ALL).map(|square| f32::from(bitboard.get_bit(square))))
        .reshape(Shape::new([8, 8]))
}

fn player_to_tensor<B: Backend>(player: &Player) -> Tensor<B, 3> {
    Tensor::stack(
        vec![
            bitboard_to_tensor(player.pawns),
            bitboard_to_tensor(player.knights),
            bitboard_to_tensor(player.bishops),
            bitboard_to_tensor(player.rooks),
            bitboard_to_tensor(player.queens),
            bitboard_to_tensor(player.king),
        ],
        0,
    )
}

fn boolean_to_tensor<B: Backend>(boolean: bool) -> Tensor<B, 2> {
    if boolean {
        Tensor::ones(Shape::new([8, 8]))
    } else {
        Tensor::zeros(Shape::new([8, 8]))
    }
}

fn board_to_tensor<B: Backend>(board: &Board) -> Tensor<B, 3> {
    Tensor::cat(
        vec![player_to_tensor(&board.us), player_to_tensor(&board.them)],
        0,
    )
}

fn final_board_to_tensor<B: Backend>(board: &Board) -> Tensor<B, 3> {
    Tensor::cat(
        vec![
            player_to_tensor(&board.us),
            player_to_tensor(&board.them),
            boolean_to_tensor(board.us.castling_rights.can_castle_king_side()).unsqueeze(),
            boolean_to_tensor(board.us.castling_rights.can_castle_queen_side()).unsqueeze(),
            boolean_to_tensor(board.them.castling_rights.can_castle_king_side()).unsqueeze(),
            boolean_to_tensor(board.them.castling_rights.can_castle_queen_side()).unsqueeze(),
            match board.playing_color {
                Color::White => Tensor::ones(Shape::new([8, 8])),
                Color::Black => Tensor::ones(Shape::new([8, 8])).neg(),
            }
            .unsqueeze(),
        ],
        0,
    )
}

pub fn boards_to_tensor<B: Backend>(boards: &[Board], move_history: usize) -> Tensor<B, 3> {
    let final_board_tensor = final_board_to_tensor(boards.last().unwrap());

    let mut board_tensors = boards[..boards.len()]
        .iter()
        .map(board_to_tensor)
        .collect::<Vec<_>>();

    board_tensors.push(final_board_tensor);
    board_tensors.insert(
        0,
        Tensor::zeros(Shape::new([
            model::calculate_board_tensor_dimension(move_history)
                - model::calculate_board_tensor_dimension(boards.len()),
            8,
            8,
        ])),
    );

    Tensor::cat(board_tensors, 0)
}

use arrayvec::ArrayVec;
use dfdx::{
    dtypes::{f16, AMP},
    losses,
    prelude::*,
};
use hash_core::{
    board::Board,
    game::{Game, Outcome},
    mg,
    repr::Player,
    BitBoard, Color, Square,
};
use std::error::Error;

use crate::model::Network;

const MAX_STEPS: usize = 100;

fn bitboard_to_tensor<D: Device<AMP<f16>>>(
    bitboard: BitBoard,
    device: &D,
) -> Tensor<Rank2<8, 8>, AMP<f16>, D> {
    device.tensor(
        Square::ALL
            .into_iter()
            .map(|index| {
                AMP(if bitboard.get_bit(index) {
                    f16::ONE
                } else {
                    f16::ZERO
                })
            })
            .collect::<Vec<_>>(),
    )
}

fn player_to_tensor<D: Device<AMP<f16>>>(
    player: &Player,
    device: &D,
) -> Tensor<Rank3<6, 8, 8>, AMP<f16>, D> {
    [
        bitboard_to_tensor(player.pawns, device),
        bitboard_to_tensor(player.knights, device),
        bitboard_to_tensor(player.bishops, device),
        bitboard_to_tensor(player.rooks, device),
        bitboard_to_tensor(player.queens, device),
        bitboard_to_tensor(player.king, device),
    ]
    .stack()
}

fn board_to_tensor<D: Device<AMP<f16>>>(
    board: &Board,
    device: &D,
) -> Tensor<Rank3<13, 8, 8>, AMP<f16>, D> {
    let mut color_tensor = match board.current_color {
        Color::White => device.ones::<Rank2<8, 8>>(),
        Color::Black => device.zeros::<Rank2<8, 8>>(),
    };

    if board.current_color == Color::Black {
        color_tensor.fill_with_zeros();
    }

    let players_tensor = (
        player_to_tensor(board.white_player(), device),
        player_to_tensor(board.black_player(), device),
    )
        .concat_along(Axis::<0>);

    (players_tensor, [color_tensor].stack()).concat_along(Axis::<0>)
}

struct TrainPoint {
    board: Board,
    evaluation: AMP<f16>,
}

// If the game wasn't decided the outcome would be a draw
struct MatchInfo {
    boards: ArrayVec<TrainPoint, MAX_STEPS>,
    outcome: Outcome,
}

fn self_play_match(network: &Network<AMP<f16>, Cpu>, device: &Cpu) -> MatchInfo {
    let mut game = Game::default();
    let mut boards = ArrayVec::<TrainPoint, MAX_STEPS>::new();

    while !boards.is_full() && game.outcome().is_none() {
        let moves = mg::gen_moves(&game.board);

        let (chosen_move, evaluation) = moves
            .iter()
            .zip(
                network
                    .forward(
                        moves
                            .iter()
                            .map(|chess_move| {
                                unsafe { game.make_move_unchecked(chess_move) };

                                let tensor = board_to_tensor(&game.board, device);

                                game.unmake_last_move();

                                tensor
                            })
                            .collect::<Vec<_>>()
                            .stack(),
                    )
                    .as_vec(),
            )
            .max_by(|(_, best), (_, eval)| best.partial_cmp(eval).unwrap())
            .unwrap();

        unsafe { game.make_move_unchecked(chosen_move) };

        print!("{chosen_move}: {evaluation} ");

        boards.push(TrainPoint {
            board: game.board,
            evaluation,
        });
    }

    let outcome = game.outcome().unwrap_or(Outcome::Draw);

    if !matches!(outcome, Outcome::Draw) {
        println!("\nwin");
    }

    MatchInfo { boards, outcome }
}

#[derive(Clone, Copy)]
pub struct Hyperparams {
    pub discount_factor: AMP<f16>,
    pub max_fitting_iterations: usize,
    pub acceptable_loss: AMP<f16>,
    pub batch_size: usize,
    pub max_games: usize,
}

#[allow(clippy::needless_borrow)]
fn fit_to_match<O: Optimizer<Network<AMP<f16>, Cpu>, Cpu, AMP<f16>>>(
    network: &mut Network<AMP<f16>, Cpu>,
    mut info: MatchInfo,
    hyperparams: Hyperparams,
    optimizer: &mut O,
    device: &Cpu,
) -> Result<(), Box<dyn Error>> {
    let mut gradients = network.try_alloc_grads()?;

    let mut discount = AMP(f16::ONE);

    info.boards.reverse();

    for _ in 0..hyperparams.max_fitting_iterations {
        let mut loss_sum = AMP(f16::from_f32(0.0));

        for boards in info.boards.chunks(hyperparams.batch_size) {
            for TrainPoint {
                board,
                evaluation: mut target_evaluation,
            } in boards
            {
                let evaluation =
                    network.try_forward_mut(board_to_tensor(&board, device).traced(gradients))?;

                target_evaluation += AMP(match info.outcome {
                    Outcome::Draw => f16::from_f32(-0.5),
                    Outcome::Win(win_color) if board.current_color == win_color => {
                        f16::from_f32(1.0)
                    }
                    _ => f16::from_f32(-1.0),
                }) * discount;

                discount *= hyperparams.discount_factor;

                let loss = losses::mse_loss(evaluation, device.tensor([target_evaluation]));
                loss_sum += loss.array();

                gradients =
                    (loss / AMP(f16::from_f32(hyperparams.batch_size as f32))).try_backward()?;
            }

            optimizer.update(network, &gradients)?;
            network.try_zero_grads(&mut gradients)?;
        }

        let average_loss = loss_sum / AMP(f16::from_f32(info.boards.len() as f32));

        println!("AVG LOSS = {average_loss}",);

        if average_loss <= hyperparams.acceptable_loss {
            break;
        }
    }

    Ok(())
}

pub fn train<O: Optimizer<Network<AMP<f16>, Cpu>, Cpu, AMP<f16>>>(
    network: &mut Network<AMP<f16>, Cpu>,
    hyperparams: Hyperparams,
    optimizer: &mut O,
    device: &Cpu,
) -> Result<(), Box<dyn Error>> {
    for game in 0..hyperparams.max_games {
        println!("ON GAME {game}");

        let result = self_play_match(network, device);

        fit_to_match(network, result, hyperparams, optimizer, device)?;
    }

    Ok(())
}

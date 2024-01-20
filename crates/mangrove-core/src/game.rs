use std::str::FromStr;

use mangrove_bootstrap::Color;

use crate::{
    board::{Board, MakeMoveError, ParseBoardError},
    mg,
    repr::ChessMove,
};

pub enum Outcome {
    Win(Color),
    Draw,
}

pub struct Game {
    board: Board,
}

impl Game {
    pub fn starting_position() -> Self {
        Self {
            board: Board::starting_position(),
        }
    }

    pub fn outcome(&self) -> Option<Outcome> {
        if mg::gen_moves(&self.board).is_empty() {
            Some(if self.board.in_check() {
                Outcome::Win(!self.board.playing_color)
            } else {
                Outcome::Draw
            })
        } else {
            None
        }
    }

    pub fn make_move(&mut self, chess_move: ChessMove) -> Result<(), MakeMoveError> {
        self.board.make_move(chess_move)
    }

    pub fn board(&self) -> &Board {
        &self.board
    }
}

impl FromStr for Game {
    type Err = ParseBoardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Board::from_str(s).map(|board| Self { board })
    }
}

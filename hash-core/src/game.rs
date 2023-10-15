use std::str::FromStr;

use hash_bootstrap::Color;

use crate::{board::Board, cache::Cache, mg, repr::Move};

const REPETITIONS: usize = 1000;

pub enum Outcome {
    Win(Color),
    Draw,
}

#[derive(PartialEq)]
enum Repetition {
    Once,
    Never,
}

pub struct Game {
    board: Board,
    repetitions: Cache<Board, Repetition, REPETITIONS>,
}

impl Game {
    pub fn starting_position() -> Self {
        Self {
            board: Board::starting_position(),
            repetitions: Cache::new(),
        }
    }

    fn was_current_board_repeated_thrice(&self) -> bool {
        if let Some(repetition) = self.repetitions.get(&self.board) {
            *repetition == Repetition::Once
        } else {
            false
        }
    }

    fn can_either_player_claim_draw(&self) -> bool {
        self.board.min_ply_clock >= 100 || self.was_current_board_repeated_thrice()
    }

    pub fn outcome(&self) -> Option<Outcome> {
        if mg::gen_moves(&self.board).is_empty() || self.can_either_player_claim_draw() {
            Some(if self.board.in_check() {
                Outcome::Win(!self.board.playing_color)
            } else {
                Outcome::Draw
            })
        } else {
            None
        }
    }

    pub unsafe fn make_move_unchecked(&mut self, chess_move: Move) {
        self.repetitions.insert(
            &self.board,
            if self.repetitions.get(&self.board).is_none() {
                Repetition::Never
            } else {
                Repetition::Once
            },
        );

        // SAFETY: Move is assumed to be legal in this position
        unsafe {
            self.board.make_move_unchecked(&chess_move);
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }
}

impl FromStr for Game {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            board: Board::from_str(s)?,
            repetitions: Cache::new(),
        })
    }
}

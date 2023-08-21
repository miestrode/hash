use std::{
    fmt::{self, Display},
    str::FromStr,
};

use crate::{
    board::Board,
    cache::Cache,
    index::{zobrist_castling_rights, zobrist_ep_file, zobrist_piece_table, zobrist_side},
    mg::{self, MOVES},
    repr::{ColoredPieceTable, EpData, Move, Piece, Player},
};

use arrayvec::ArrayVec;
use hash_build::{Color, Square};

pub enum Outcome {
    BlackWin,
    WhiteWin,
    Draw,
}

const CACHE_LENGTH: usize = 200;

#[derive(Clone, Copy)]
pub struct Game {
    pub board: Board,
    half_moves: u16, // This is the number of half moves since the last capture or pawn move
    repetition_cache: Cache<u8, CACHE_LENGTH>,
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board && self.half_moves == other.half_moves
    }
}

impl FromStr for Game {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(' ').collect::<Vec<_>>();

        if parts.len() != 6 {
            Err("Input must contain 6 parts separated by spaces")
        } else {
            let colored_piece_table = ColoredPieceTable::from_str(parts[0])?;
            let current_color = Color::from_str(parts[1])?;

            let ep_data = match parts[3] {
                "-" => None,
                square => Some({
                    let capture_point = Square::from_str(square)?.as_bitboard();
                    EpData {
                        capture_point,
                        pawn: capture_point
                            .move_one_down(current_color)
                            .first_one_as_square(),
                    }
                }),
            };

            let half_moves = parts[4]
                .parse::<u16>()
                .map_err(|_| "Input contains invalid number for half-moves")?;

            if parts[5].parse::<u16>().is_err() {
                return Err("Input contains invalid number for full-moves");
            }

            let mut white = Player::blank();
            let mut black = Player::blank();

            for (square_index, piece) in colored_piece_table.0.into_iter().enumerate() {
                if let Some(Piece {
                    kind,
                    color: Color::White,
                }) = piece
                {
                    white.toggle_piece(kind, Square(square_index as u32));
                } else if let Some(Piece {
                    kind,
                    color: Color::Black,
                }) = piece
                {
                    black.toggle_piece(kind, Square(square_index as u32));
                }
            }

            if white.king.is_empty() || black.king.is_empty() {
                return Err("Input is illegal as a FEN-string must include both players' kings");
            }

            if parts[2] != "-" {
                white.castling_rights.0[Square::H1] = parts[2].contains('K');
                white.castling_rights.0[Square::A1] = parts[2].contains('Q');
                black.castling_rights.0[Square::H8] = parts[2].contains('k');
                black.castling_rights.0[Square::A8] = parts[2].contains('q');
                // This would indicate the part contains some characters other than K, Q, k or q.
                if ((white.castling_rights.0[Square::H1] as usize)
                    + (white.castling_rights.0[Square::A1] as usize)
                    + (black.castling_rights.0[Square::H8] as usize)
                    + (black.castling_rights.0[Square::A8] as usize))
                    != parts[2].len()
                {
                    return Err("Input contains invalid data for castling information");
                }
            }

            white.castling_rights.0[Square::E1] = true;
            black.castling_rights.0[Square::E8] = true;

            let (current_player, opposing_player) = match current_color {
                Color::White => (white, black),
                Color::Black => (black, white),
            };

            let board = Board {
                current_player,
                opposing_player,
                current_color,
                piece_table: colored_piece_table.uncolored(),
                ep_data,
                // the FEN-string is assumed to be valid
                hash: unsafe { zobrist_piece_table(&colored_piece_table) }
                    ^ zobrist_side(current_color)
                    ^ ep_data.map_or(0, |ep_data| zobrist_ep_file(ep_data.pawn.file()))
                    ^ zobrist_castling_rights(&white.castling_rights)
                    ^ zobrist_castling_rights(&black.castling_rights),
            };

            let mut game = Self {
                board,
                half_moves,
                // TODO: Experiment with differing values find the optimal values
                // for this assignment
                repetition_cache: Cache::new(),
            };

            game.board.update_move_constraints();

            Ok(game)
        }
    }
}

impl Display for Game {
    // TODO: Refactor this to look nicer
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        {
            for column in (0..8).rev() {
                let mut spacing = 0;

                for row in 0..8 {
                    let square = Square(column * 8 + row);
                    let piece = self.board.get_piece(square);

                    if let Some(piece) = piece {
                        if spacing != 0 {
                            spacing.fmt(f)?;
                        }

                        piece.fmt(f)?;
                        spacing = 0;
                    } else {
                        spacing += 1;
                    }
                }

                if spacing != 0 {
                    spacing.fmt(f)?;
                }

                if column != 0 {
                    '/'.fmt(f)?;
                }
            }
        }

        ' '.fmt(f)?;

        self.board.current_color.fmt(f)?;

        ' '.fmt(f)?;

        let (white, black) = match self.board.current_color {
            Color::White => (self.board.current_player, self.board.opposing_player),
            Color::Black => (self.board.opposing_player, self.board.current_player),
        };

        {
            let mut castling_string = String::new();

            if white.castling_rights.can_castle_ks() {
                castling_string.push('K');
            }

            if white.castling_rights.can_castle_qs() {
                castling_string.push('Q');
            }

            if black.castling_rights.can_castle_ks() {
                castling_string.push('k');
            }

            if black.castling_rights.can_castle_qs() {
                castling_string.push('q');
            }

            if castling_string.is_empty() {
                castling_string.push('-')
            }

            castling_string.fmt(f)?;
        }

        ' '.fmt(f)?;

        if let Some(ep_data) = self.board.ep_data {
            ep_data.capture_point.first_one_as_square().fmt(f)?;
        } else {
            '-'.fmt(f)?;
        }
        f.write_fmt(format_args!(" {} 1", self.half_moves))
    }
}

impl Default for Game {
    fn default() -> Self {
        // Taken from https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation
        Self::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}

impl Game {
    // SAFETY: The move is assumed to be legal
    pub unsafe fn make_move_unchecked(&mut self, chess_move: &Move) {
        if unsafe { self.board.make_move_unchecked(chess_move) } {
            self.half_moves += 1;
        }
    }

    pub fn following_games(&self) -> ArrayVec<(Move, Game), MOVES> {
        mg::gen_moves(&self.board)
            .into_iter()
            .map(|chess_move| {
                let mut game = *self;

                unsafe {
                    game.make_move_unchecked(&chess_move);
                }

                (chess_move, game)
            })
            .collect()
    }

    // NOTE: that draw by repetition and the 50-move rule both require one to claim the draw
    // (although in practice, it is autoclaimed by the GUI. Despite that, mate-in-one issues strip
    // of them the right to be here)
    pub fn outcome(&self) -> Option<Outcome> {
        if mg::gen_moves(&self.board).is_empty() {
            // If a player is in check, he is attacked and so this is mate. The player who is
            // moving thus lost
            if self.board.current_player.is_in_check() {
                Some(match self.board.current_color {
                    Color::White => Outcome::BlackWin,
                    Color::Black => Outcome::WhiteWin,
                })
            } else {
                // Otherwise, it's stalemate
                Some(Outcome::Draw)
            }
        } else {
            None
        }
    }

    pub fn was_repeated_thrice(&self, board: &Board) -> bool {
        self.repetition_cache.get(board) == Some(3)
    }

    // Can either player claim a draw in this position?
    pub fn can_claim_draw(&self) -> bool {
        // Credible source: https://www.chessprogramming.org/Fifty-move_Rule
        self.half_moves >= 100 && self.was_repeated_thrice(&self.board)
    }

    pub fn half_moves(&self) -> u16 {
        self.half_moves
    }
}

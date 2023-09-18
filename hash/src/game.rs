use std::{
    fmt::{self, Display},
    str::FromStr,
};

use crate::{
    board::Board,
    cache::Cache,
    index::zobrist,
    mg::{self},
    repr::{ColoredPieceTable, Move, Piece, Player},
};

use hash_bootstrap::{BitBoard, Color, Square};

pub enum Outcome {
    Win(Color),
    Draw,
}

const CACHE_LENGTH: usize = 1000;

#[derive(Clone)]
pub struct Game {
    pub board: Board,
    half_moves: u16, // This is the number of half moves since the last capture or pawn move
    repetition_cache: Cache<u8, CACHE_LENGTH>,
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

            let en_passant_capture_square = match parts[3] {
                "-" => None,
                square => Some(Square::from_str(square)?.into()),
            };

            let half_moves = parts[4]
                .parse::<u16>()
                .map_err(|_| "Input contains invalid number for half-moves")?;

            if parts[5].parse::<u16>().is_err() {
                return Err("Input contains invalid number for full-moves");
            }

            let mut white = Player::blank();
            let mut black = Player::blank();

            for (piece, square) in colored_piece_table
                .pieces()
                .iter()
                .copied()
                .zip(Square::ALL)
            {
                if let Some(Piece {
                    kind,
                    color: Color::White,
                }) = piece
                {
                    white.toggle_piece(square, kind);
                } else if let Some(Piece {
                    kind,
                    color: Color::Black,
                }) = piece
                {
                    black.toggle_piece(square, kind);
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
                us: current_player,
                them: opposing_player,
                playing_color: current_color,
                piece_table: colored_piece_table.uncolored(),
                en_passant_capture_square,
                hash: zobrist::piece_table(&colored_piece_table)
                    ^ zobrist::side(current_color)
                    ^ en_passant_capture_square
                        .map_or(0, |square| zobrist::en_passant_file(square.file()))
                    ^ zobrist::castling_rights(&white.castling_rights)
                    ^ zobrist::castling_rights(&black.castling_rights),
                checkers: BitBoard::EMPTY,
                pinned: BitBoard::EMPTY,
            };

            let mut game = Self {
                board,
                half_moves,
                repetition_cache: Cache::new(),
            };

            game.board.update_move_restrictions();

            Ok(game)
        }
    }
}

impl Display for Game {
    // TODO: Refactor this to look nicer
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        {
            for column in (0..8).rev() {
                let mut spacing = 0;

                for row in 0..8 {
                    let square = Square::try_from(column * 8 + row).unwrap();
                    let piece = self.board.piece(square);

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

        self.board.playing_color.fmt(f)?;

        ' '.fmt(f)?;

        let (white, black) = match self.board.playing_color {
            Color::White => (self.board.us, self.board.them),
            Color::Black => (self.board.them, self.board.us),
        };

        {
            let mut castling_string = String::new();

            if white.castling_rights.can_castle_king_side() {
                castling_string.push('K');
            }

            if white.castling_rights.can_castle_queen_side() {
                castling_string.push('Q');
            }

            if black.castling_rights.can_castle_king_side() {
                castling_string.push('k');
            }

            if black.castling_rights.can_castle_queen_side() {
                castling_string.push('q');
            }

            if castling_string.is_empty() {
                castling_string.push('-')
            }

            castling_string.fmt(f)?;
        }

        ' '.fmt(f)?;

        if let Some(square) = self.board.en_passant_capture_square {
            square.fmt(f)?;
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
    pub fn perft(&self, depth: u32) -> u64 {
        fn perft(board: &Board, depth: u32) -> u64 {
            let moves = mg::gen_moves(board);

            match depth {
                // At a depth of one we know all next moves will reach depth zero.
                // Thus, we can know they are all leaves and add one each to the nodes searched.
                1 => moves.len() as u64,
                _ => moves
                    .into_iter()
                    .map(|chess_move| {
                        let mut new_board = *board;
                        unsafe { new_board.make_move_unchecked(&chess_move) };

                        perft(&new_board, depth - 1)
                    })
                    .sum(),
            }
        }

        perft(&self.board, depth)
    }

    pub(crate) unsafe fn make_move_unchecked(&mut self, chess_move: &Move) {
        let previous_value = self.repetition_cache.get(&self.board).unwrap_or(0);
        self.repetition_cache
            .insert(&self.board, previous_value + 1);

        if unsafe { self.board.make_move_unchecked(chess_move) } {
            self.half_moves += 1;
        }
    }

    // NOTE: that draw by repetition and the 50-move rule both require one to claim the draw
    // (although in practice, it is auto claimed by the GUI. Despite that, mate-in-one issues strip
    // of them the right to be here)
    pub fn outcome(&self) -> Option<Outcome> {
        if mg::gen_moves(&self.board).is_empty() {
            // If a player is in check, he is attacked and so this is mate. The player who is
            // moving thus lost
            if self.board.in_check() {
                Some(Outcome::Win(self.board.playing_color))
            } else {
                // Otherwise, it's stalemate
                Some(Outcome::Draw)
            }
        } else {
            None
        }
    }

    fn was_repeated_thrice(&self, board: &Board) -> bool {
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

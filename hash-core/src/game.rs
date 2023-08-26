use std::{
    fmt::{self, Display},
    mem,
    str::FromStr,
};

use crate::{
    board::Board,
    cache::Cache,
    index::{zobrist_castling_rights, zobrist_ep_file, zobrist_piece_table, zobrist_side},
    mg::{self},
    repr::{ColoredPieceTable, EpData, Move, MoveMeta, Piece, PieceKind, Pins, Player},
};

use hash_build::{BitBoard, Color, Square};

pub enum Outcome {
    BlackWin,
    WhiteWin,
    Draw,
}

struct RestorationData {
    current_origin_castling_right: bool,
    opposing_target_castling_right: bool,
    opposing_king_must_move: bool,
    opposing_pins: Pins,
    opposing_valid_targets: BitBoard,
    captured_piece_kind: Option<PieceKind>,
    ep_data: Option<EpData>,
    board_hash: u64,
    applied_move: Move,
    half_moves: u16, // This is the number of half moves since the last capture or pawn move
}

const CACHE_LENGTH: usize = 1000;
const RESTORATION_DATA_START_CAPACITY: usize = 8;

pub struct Game {
    pub board: Board,
    half_moves: u16, // This is the number of half moves since the last capture or pawn move
    repetition_cache: Cache<u8, CACHE_LENGTH>,
    // TODO: Consider using an array here:
    restoration_data: Vec<RestorationData>,
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
                restoration_data: Vec::with_capacity(RESTORATION_DATA_START_CAPACITY),
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
        self.restoration_data.push(RestorationData {
            current_origin_castling_right: self.board.current_player.castling_rights.0
                [chess_move.origin],
            opposing_target_castling_right: self.board.opposing_player.castling_rights.0
                [chess_move.target],
            opposing_king_must_move: self.board.opposing_player.king_must_move,
            opposing_pins: self.board.opposing_player.pins,
            opposing_valid_targets: self.board.opposing_player.valid_targets,
            // If the move is an en-passant, this isn't used, and so this need not be fully
            // accurate.
            captured_piece_kind: self.board.piece_table.piece_kind(chess_move.target),
            ep_data: self.board.ep_data,
            board_hash: self.board.hash,
            applied_move: *chess_move,
            half_moves: self.half_moves,
        });

        let previous_value = self.repetition_cache.get(&self.board).unwrap_or(0);
        self.repetition_cache
            .insert(&self.board, previous_value + 1);

        if unsafe { self.board.make_move_unchecked(chess_move) } {
            self.half_moves += 1;
        }
    }

    /// Undo the last move on this game. If there was no move to undo, `false` is returned and
    /// otherwise `true` is.
    pub fn unmake_last_move(&mut self) -> bool {
        if let Some(restoration_data) = self.restoration_data.pop() {
            self.half_moves = restoration_data.half_moves;
            self.board.hash = restoration_data.board_hash;

            self.board.piece_table.move_piece(
                restoration_data.applied_move.target,
                restoration_data.applied_move.origin,
            );
            self.board.current_color = !self.board.current_color;
            mem::swap(
                &mut self.board.current_player,
                &mut self.board.opposing_player,
            );
            self.board.ep_data = restoration_data.ep_data;

            if let MoveMeta::Promotion(piece_kind) = restoration_data.applied_move.meta {
                self.board.current_player.toggle_piece(
                    restoration_data.applied_move.moved_piece_kind,
                    restoration_data.applied_move.origin,
                );
                self.board
                    .current_player
                    .toggle_piece(piece_kind, restoration_data.applied_move.target);
            } else {
                unsafe {
                    self.board.current_player.move_piece_unchecked(
                        restoration_data.applied_move.moved_piece_kind,
                        restoration_data.applied_move.target,
                        restoration_data.applied_move.origin,
                    )
                };
            }

            if let MoveMeta::EnPassant = restoration_data.applied_move.meta {
                if let Some(ep_data) = restoration_data.ep_data {
                    self.board
                        .opposing_player
                        .toggle_piece(PieceKind::Pawn, ep_data.pawn);
                    self.board
                        .piece_table
                        .set(Some(PieceKind::Pawn), ep_data.pawn);
                } else {
                    unreachable!()
                }
            } else if let Some(piece_kind) = restoration_data.captured_piece_kind {
                self.board
                    .opposing_player
                    .toggle_piece(piece_kind, restoration_data.applied_move.target);
                self.board.piece_table.set(
                    restoration_data.captured_piece_kind,
                    restoration_data.applied_move.target,
                );
            }

            self.board.current_player.castling_rights.0[restoration_data.applied_move.origin] =
                restoration_data.current_origin_castling_right;
            self.board.opposing_player.castling_rights.0[restoration_data.applied_move.target] =
                restoration_data.opposing_target_castling_right;

            match restoration_data.applied_move.meta {
                MoveMeta::CastleKs => match self.board.current_color {
                    Color::White => unsafe {
                        self.board.current_player.move_piece_unchecked(
                            PieceKind::Rook,
                            Square::F1,
                            Square::BOTTOM_RIGHT_ROOK,
                        );
                        self.board
                            .piece_table
                            .move_piece(Square::F1, Square::BOTTOM_RIGHT_ROOK);
                    },
                    Color::Black => unsafe {
                        self.board.current_player.move_piece_unchecked(
                            PieceKind::Rook,
                            Square::F8,
                            Square::TOP_RIGHT_ROOK,
                        );
                        self.board
                            .piece_table
                            .move_piece(Square::F8, Square::TOP_RIGHT_ROOK);
                    },
                },
                MoveMeta::CastleQs => match self.board.current_color {
                    Color::White => unsafe {
                        self.board.current_player.move_piece_unchecked(
                            PieceKind::Rook,
                            Square::C1,
                            Square::BOTTOM_LEFT_ROOK,
                        );
                        self.board
                            .piece_table
                            .move_piece(Square::C1, Square::BOTTOM_LEFT_ROOK);
                    },
                    Color::Black => unsafe {
                        self.board.current_player.move_piece_unchecked(
                            PieceKind::Rook,
                            Square::C8,
                            Square::TOP_LEFT_ROOK,
                        );
                        self.board
                            .piece_table
                            .move_piece(Square::C8, Square::TOP_LEFT_ROOK);
                    },
                },
                _ => {}
            }

            self.board.opposing_player.king_must_move = restoration_data.opposing_king_must_move;
            self.board.opposing_player.pins = restoration_data.opposing_pins;
            self.board.opposing_player.valid_targets = restoration_data.opposing_valid_targets;

            // TODO:: Add utility methods to the cache API, to make this code nicer
            if let Some(value) = self.repetition_cache.get(&self.board) {
                self.repetition_cache.insert(&self.board, value - 1);
            }

            true
        } else {
            false
        }
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

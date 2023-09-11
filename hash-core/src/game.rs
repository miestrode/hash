use std::{
    fmt::{self, Display},
    mem,
    str::FromStr,
};

use crate::{
    board::Board,
    cache::Cache,
    mg::{self},
    repr::{ColoredPieceTable, EnPassantData, Move, MoveMetadata, Piece, PieceKind, Pins, Player},
};

use hash_bootstrap::{BitBoard, Color, Square};

pub enum Outcome {
    Win(Color),
    Draw,
}

#[derive(Clone, Copy)]
struct RestorationData {
    current_origin_castling_right: bool,
    opposing_target_castling_right: bool,
    opposing_king_must_move: bool,
    opposing_pins: Pins,
    opposing_valid_targets: BitBoard,
    captured_piece_kind: Option<PieceKind>,
    ep_data: Option<EnPassantData>,
    board_hash: u64,
    applied_move: Move,
    half_moves: u16, // This is the number of half moves since the last capture or pawn move
}

const CACHE_LENGTH: usize = 1000;
const RESTORATION_DATA_START_CAPACITY: usize = 8;

#[derive(Clone)]
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
                    EnPassantData {
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
                us: current_player,
                them: opposing_player,
                playing_color: current_color,
                piece_table: colored_piece_table.uncolored(),
                en_passant_data: ep_data,
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

        if let Some(ep_data) = self.board.en_passant_data {
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
    pub fn perft(&mut self, depth: u32) -> u64 {
        let moves = mg::gen_moves(&self.board);

        match depth {
            // At a depth of one we know all next moves will reach depth zero.
            // Thus, we can know they are all leaves and add one each to the nodes searched.
            1 => moves.len() as u64,
            _ => moves
                .into_iter()
                .map(|chess_move| {
                    unsafe { self.make_move_unchecked(&chess_move) };
                    let result = self.perft(depth - 1);
                    self.unmake_last_move();

                    result
                })
                .sum(),
        }
    }

    // SAFETY: The move is assumed to be legal
    pub unsafe fn make_move_unchecked(&mut self, chess_move: &Move) {
        self.restoration_data.push(RestorationData {
            current_origin_castling_right: self.board.us.castling_rights.0[chess_move.origin],
            opposing_target_castling_right: self.board.them.castling_rights.0[chess_move.target],
            opposing_king_must_move: self.board.them.king_must_move,
            opposing_pins: self.board.them.pins,
            opposing_valid_targets: self.board.them.valid_targets,
            // If the move is an en-passant, this isn't used, and so this need not be fully
            // accurate.
            captured_piece_kind: self.board.piece_table.piece_kind(chess_move.target),
            ep_data: self.board.en_passant_data,
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
            self.board.playing_color = !self.board.playing_color;
            mem::swap(&mut self.board.us, &mut self.board.them);
            self.board.en_passant_data = restoration_data.ep_data;

            if let MoveMetadata::Promotion(piece_kind) = restoration_data.applied_move.metadata {
                self.board
                    .us
                    .toggle_piece(PieceKind::Pawn, restoration_data.applied_move.origin);
                self.board
                    .us
                    .toggle_piece(piece_kind, restoration_data.applied_move.target);
                self.board
                    .piece_table
                    .set(Some(PieceKind::Pawn), restoration_data.applied_move.origin);
            } else {
                unsafe {
                    self.board.us.move_piece_unchecked(
                        restoration_data.applied_move.piece_kind,
                        restoration_data.applied_move.target,
                        restoration_data.applied_move.origin,
                    )
                };
            }

            if let MoveMetadata::EnPassant = restoration_data.applied_move.metadata {
                if let Some(ep_data) = restoration_data.ep_data {
                    self.board.them.toggle_piece(PieceKind::Pawn, ep_data.pawn);
                    self.board
                        .piece_table
                        .set(Some(PieceKind::Pawn), ep_data.pawn);
                } else {
                    unreachable!()
                }
            } else if let Some(piece_kind) = restoration_data.captured_piece_kind {
                self.board
                    .them
                    .toggle_piece(piece_kind, restoration_data.applied_move.target);
                self.board.piece_table.set(
                    restoration_data.captured_piece_kind,
                    restoration_data.applied_move.target,
                );
            }

            self.board.us.castling_rights.0[restoration_data.applied_move.origin] =
                restoration_data.current_origin_castling_right;
            self.board.them.castling_rights.0[restoration_data.applied_move.target] =
                restoration_data.opposing_target_castling_right;

            match restoration_data.applied_move.metadata {
                MoveMetadata::CastleKingSide => match self.board.playing_color {
                    Color::White => unsafe {
                        self.board.us.move_piece_unchecked(
                            PieceKind::Rook,
                            Square::F1,
                            Square::BOTTOM_RIGHT_ROOK,
                        );
                        self.board
                            .piece_table
                            .move_piece(Square::F1, Square::BOTTOM_RIGHT_ROOK);
                    },
                    Color::Black => unsafe {
                        self.board.us.move_piece_unchecked(
                            PieceKind::Rook,
                            Square::F8,
                            Square::TOP_RIGHT_ROOK,
                        );
                        self.board
                            .piece_table
                            .move_piece(Square::F8, Square::TOP_RIGHT_ROOK);
                    },
                },
                MoveMetadata::CastleQueenSide => match self.board.playing_color {
                    Color::White => unsafe {
                        self.board.us.move_piece_unchecked(
                            PieceKind::Rook,
                            Square::D1,
                            Square::BOTTOM_LEFT_ROOK,
                        );
                        self.board
                            .piece_table
                            .move_piece(Square::D1, Square::BOTTOM_LEFT_ROOK);
                    },
                    Color::Black => unsafe {
                        self.board.us.move_piece_unchecked(
                            PieceKind::Rook,
                            Square::D8,
                            Square::TOP_LEFT_ROOK,
                        );
                        self.board
                            .piece_table
                            .move_piece(Square::D8, Square::TOP_LEFT_ROOK);
                    },
                },
                _ => {}
            }

            self.board.them.king_must_move = restoration_data.opposing_king_must_move;
            self.board.them.pins = restoration_data.opposing_pins;
            self.board.them.valid_targets = restoration_data.opposing_valid_targets;

            // TODO:: Add utility methods to the cache API, to make this code nicer
            if let Some(value) = self.repetition_cache.get(&self.board) {
                self.repetition_cache
                    .insert(&self.board, value.saturating_sub(1));
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
            if self.board.us.is_in_check() {
                Some(Outcome::Win(self.board.playing_color))
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

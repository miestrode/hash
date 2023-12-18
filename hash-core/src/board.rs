use std::{
    fmt,
    fmt::Display,
    mem,
    num::{IntErrorKind, ParseIntError},
    str::FromStr,
};

use crate::{
    index,
    index::zobrist,
    mg,
    repr::{ChessMove, ParsePieceBoardError, Piece, PieceBoard, PieceKind, PieceKindBoard, Player},
};
use hash_bootstrap::{BitBoard, Color, ParseSquareError, Square};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Board {
    pub us: Player,
    pub them: Player,
    pub checkers: BitBoard,
    pub pinned: BitBoard,
    pub playing_color: Color,
    pub en_passant_capture_square: Option<Square>,
    pub piece_table: PieceKindBoard,
    pub min_ply_clock: u8,
    pub full_moves: u16,
    pub hash: u64,
}

impl Board {
    pub fn starting_position() -> Self {
        Self::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn is_attacked(&self, square: Square) -> bool {
        let mut attackers = BitBoard::EMPTY;
        let occupation = self.occupation() - self.us.king;

        attackers += index::rook_slides(square, occupation) & (self.them.rooks + self.them.queens);
        attackers +=
            index::bishop_slides(square, occupation) & (self.them.bishops + self.them.queens);

        attackers += index::knight_attacks(square) & self.them.knights;
        attackers += index::king_attacks(square) & self.them.king;

        let square: BitBoard = square.into();

        attackers += (square.move_one_up_left(self.playing_color)
            + square.move_one_up_right(self.playing_color))
            & self.them.pawns;

        !attackers.is_empty()
    }

    pub fn in_check(&self) -> bool {
        !self.checkers.is_empty()
    }

    pub fn occupation(&self) -> BitBoard {
        self.us.occupation + self.them.occupation
    }

    pub fn piece(&self, square: Square) -> Option<Piece> {
        self.piece_table.piece_kind(square).map(|kind| Piece {
            kind,
            color: if self.us.occupation.get_bit(square) {
                self.playing_color
            } else {
                !self.playing_color
            },
        })
    }

    // INVARIANT: A piece as specified must NOT exist on the specified square.
    unsafe fn add_piece_unchecked(&mut self, square: Square, piece: Piece) {
        self.piece_table.set(square, Some(piece.kind));

        if piece.color == self.playing_color {
            &mut self.us
        } else {
            &mut self.them
        }
        .toggle_piece(square, piece.kind);
    }

    // INVARIANT: A piece as specified must exist on the specified square.
    unsafe fn remove_piece_unchecked(&mut self, square: Square, piece: Piece) {
        self.piece_table.set(square, None);

        if piece.color == self.playing_color {
            &mut self.us
        } else {
            &mut self.them
        }
        .toggle_piece(square, piece.kind);
    }

    pub fn update_move_restrictions(&mut self) {
        // SAFETY: The board is assumed to be validly constructed
        let king_square = unsafe { Square::try_from(self.us.king).unwrap_unchecked() };

        self.checkers ^= index::knight_attacks(king_square) & self.them.knights;
        self.checkers ^= index::pawn_attacks(king_square, self.playing_color) & self.them.pawns;

        // Get all the sliding pieces that could be attacking the king
        let attackers = (index::rook_slides(king_square, self.them.occupation)
            & (self.them.rooks + self.them.queens))
            + (index::bishop_slides(king_square, self.them.occupation)
                & (self.them.bishops + self.them.queens));

        // Update pins
        for attacker in attackers.bits() {
            let pieces_between = index::line_between(attacker, king_square) & self.us.occupation;

            if pieces_between.is_empty() {
                self.checkers ^= attacker.into();
            } else if pieces_between.is_a_single_one() {
                self.pinned ^= pieces_between;
            }
        }
    }

    // INVARIANT: The passed move must be legal in relation to the current board.
    pub unsafe fn make_move_unchecked(&mut self, chess_move: &ChessMove) {
        self.en_passant_capture_square = None;
        self.checkers = BitBoard::EMPTY;
        self.pinned = BitBoard::EMPTY;

        // SAFETY: The board is assumed to be valid
        let enemy_king_square = unsafe { Square::try_from(self.them.king).unwrap_unchecked() };
        let moved_piece_kind = self.piece_table.piece_kind(chess_move.origin).unwrap();
        let target_piece_kind = self.piece_table.piece_kind(chess_move.target);

        let mut is_capture = false;

        self.us.castling_rights[chess_move.origin] = false;
        self.them.castling_rights[chess_move.target] = false;

        // SAFETY: Move is assumed to be legal.
        unsafe {
            self.remove_piece_unchecked(
                chess_move.origin,
                Piece {
                    kind: moved_piece_kind,
                    color: self.playing_color,
                },
            );

            // Handle removing the captured piece
            if let Some(target_piece_kind) = target_piece_kind {
                self.remove_piece_unchecked(
                    chess_move.target,
                    Piece {
                        kind: target_piece_kind,
                        color: !self.playing_color,
                    },
                );

                is_capture = true;
            } else if moved_piece_kind == PieceKind::Pawn {
                if chess_move.origin.rank().abs_diff(chess_move.target.rank()) == 2 {
                    // This must mean the move was a double-push
                    self.en_passant_capture_square = Some(
                        chess_move
                            .target
                            .move_one_down_unchecked(self.playing_color),
                    )
                } else if chess_move.origin.file() != chess_move.target.file() {
                    // If we are here, this must mean the move was an en passant.
                    self.remove_piece_unchecked(
                        chess_move
                            .target
                            .move_one_down_unchecked(self.playing_color),
                        Piece {
                            kind: PieceKind::Pawn,
                            color: !self.playing_color,
                        },
                    );

                    is_capture = true;
                }
            }

            self.add_piece_unchecked(
                chess_move.target,
                Piece {
                    kind: chess_move.promotion.unwrap_or(moved_piece_kind),
                    color: self.playing_color,
                },
            );
        }

        let move_bitboard = BitBoard::from(chess_move.origin) ^ chess_move.target.into();

        // TODO: Check if replacing this with a more rudimentary check would be faster
        if moved_piece_kind == PieceKind::King && move_bitboard <= BitBoard::KING_CASTLE_MOVES {
            // This must mean the move was a castle.
            let king_square = chess_move.origin.as_index() as u8;

            let (origin, target) = if chess_move.target.file() == Square::G_FILE {
                (king_square + 3, king_square + 1)
            } else {
                (king_square - 4, king_square - 1)
            };

            // SAFETY: Move is assumed to be legal.
            unsafe {
                self.remove_piece_unchecked(
                    Square::try_from(origin).unwrap_unchecked(),
                    Piece {
                        kind: PieceKind::Rook,
                        color: self.playing_color,
                    },
                );

                self.add_piece_unchecked(
                    Square::try_from(target).unwrap_unchecked(),
                    Piece {
                        kind: PieceKind::Rook,
                        color: self.playing_color,
                    },
                );
            }
        }

        // Update `checkers` for the non-sliding pieces
        self.checkers ^= match self.piece_table.piece_kind(chess_move.target).unwrap() {
            PieceKind::Knight => index::knight_attacks(enemy_king_square) & self.us.knights,
            PieceKind::Pawn => {
                index::pawn_attacks(enemy_king_square, !self.playing_color) & self.us.pawns
            }
            _ => BitBoard::EMPTY,
        };

        // Get all the sliding pieces that could be attacking the enemy king
        let attackers = (index::rook_slides(enemy_king_square, self.us.occupation)
            & (self.us.rooks + self.us.queens))
            + (index::bishop_slides(enemy_king_square, self.us.occupation)
                & (self.us.bishops + self.us.queens));

        // Update pins
        for attacker in attackers.bits() {
            let pieces_between =
                index::line_between(attacker, enemy_king_square) & self.them.occupation;

            if pieces_between.is_empty() {
                self.checkers ^= attacker.into();
            } else if pieces_between.is_a_single_one() {
                self.pinned ^= pieces_between;
            }
        }

        mem::swap(&mut self.us, &mut self.them);

        self.full_moves += (self.playing_color == Color::Black) as u16;

        self.min_ply_clock = if moved_piece_kind == PieceKind::Pawn || is_capture {
            0
        } else {
            self.min_ply_clock.saturating_add(1)
        };

        self.playing_color = !self.playing_color;
    }

    pub fn perft(&self, depth: u32) -> u64 {
        let moves = mg::gen_moves(self);

        match depth {
            0 => 1,
            // At a depth of one we know all next moves will reach depth zero.
            // Thus, we can know they are all leaves and add one each to the nodes searched.
            1 => moves.len() as u64,
            _ => moves
                .into_iter()
                .map(|chess_move| {
                    let mut new_board = *self;

                    unsafe { new_board.make_move_unchecked(&chess_move) };

                    new_board.perft(depth - 1)
                })
                .sum(),
        }
    }

    pub fn gen_child_boards(&self) -> impl Iterator<Item = (ChessMove, Board)> + '_ {
        mg::gen_moves(self).into_iter().map(|chess_move| {
            let mut new_board = *self;
            unsafe {
                new_board.make_move_unchecked(&chess_move);
            };

            (chess_move, new_board)
        })
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum ParseBoardError {
    #[error("fen should contain 6 space-separated parts")]
    InvalidPartAmount,
    #[error("board setup is malformed")]
    MalformedArrangement(#[source] ParsePieceBoardError),
    #[error("color must be `w` or `b`")]
    InvalidColor,
    #[error("invalid en passant square")]
    InvalidEnPassantSquare(#[source] Option<ParseSquareError>),
    #[error("castling rights should only contain `K`, `Q`, `k`, and `q`, and at most once")]
    InvalidCastlingRights,
    #[error("half-move clock should be a non-negative integer")]
    InvalidHalfMoveClock(#[source] ParseIntError),
    #[error("full-move number should be a non-negative integer, not smaller than the half-move clock over 2, floored")]
    InvalidFullMoveNumber(#[source] Option<ParseIntError>),
    #[error("provided board allows for the capture of a king")]
    CapturableKing,
    #[error("provided board has pawns on edge ranks")]
    PawnsOnEdgeRanks,
    #[error("provided board must have one king for each side")]
    InvalidKingCount,
}

// TODO: Check that the board is logical, meaning we have two kings, no pawns are on the edge
// ranks, the en passant square is possible, and the playing side cannot capture the opponent's king.
impl FromStr for Board {
    type Err = ParseBoardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(' ').collect::<Vec<_>>();

        if parts.len() != 6 {
            Err(ParseBoardError::InvalidPartAmount)
        } else {
            let piece_board =
                PieceBoard::from_str(parts[0]).map_err(ParseBoardError::MalformedArrangement)?;
            let current_color =
                Color::from_str(parts[1]).map_err(|_| ParseBoardError::InvalidColor)?;

            let en_passant_capture_square = match parts[3] {
                "-" => None,
                square => Some(
                    Square::from_str(square)
                        .map_err(|error| ParseBoardError::InvalidEnPassantSquare(Some(error)))?,
                ),
            };

            let ply_clock = match parts[4].parse::<u8>() {
                Ok(ply_clock) => ply_clock,
                Err(error) if *error.kind() == IntErrorKind::PosOverflow => u8::MAX,
                Err(error) => return Err(ParseBoardError::InvalidHalfMoveClock(error)),
            };

            let full_moves = match parts[5].parse::<u16>() {
                Ok(full_moves) if full_moves >= ply_clock as u16 / 2 => full_moves,
                Ok(_) => return Err(ParseBoardError::InvalidFullMoveNumber(None)),
                Err(error) if *error.kind() == IntErrorKind::PosOverflow => u16::MAX,
                Err(error) => return Err(ParseBoardError::InvalidFullMoveNumber(Some(error))),
            };

            let mut white = Player::blank();
            let mut black = Player::blank();

            for (piece, square) in piece_board.pieces().iter().copied().zip(Square::ALL) {
                if let Some(Piece { kind, color }) = piece {
                    match color {
                        Color::White => &mut white,
                        Color::Black => &mut black,
                    }
                    .toggle_piece(square, kind);
                }
            }

            // When creating a board from FEN, the rook squares encode castling, and since
            // CastlingRights uses the conjunction of the king and rook squares, they are assigned
            // `true` here.
            white.castling_rights[Square::E1] = true;
            black.castling_rights[Square::E8] = true;

            if parts[2] != "-" {
                white.castling_rights[Square::H1] = parts[2].contains('K');
                white.castling_rights[Square::A1] = parts[2].contains('Q');
                black.castling_rights[Square::H8] = parts[2].contains('k');
                black.castling_rights[Square::A8] = parts[2].contains('q');

                // Assuming each match was singular and the part has no invalid characters,
                // this would be the length of the part.
                let assumed_length = (white.castling_rights[Square::H1] as usize)
                    + (white.castling_rights[Square::A1] as usize)
                    + (black.castling_rights[Square::H8] as usize)
                    + (black.castling_rights[Square::A8] as usize);

                // If the assumed length is not the actual one, the part has some invalid
                // characters, or repetitions
                if parts[2].is_empty() || assumed_length != parts[2].len() {
                    return Err(ParseBoardError::InvalidCastlingRights);
                }
            }

            let (current_player, opposing_player) = match current_color {
                Color::White => (white, black),
                Color::Black => (black, white),
            };

            let mut board = Board {
                us: current_player,
                them: opposing_player,
                playing_color: current_color,
                piece_table: piece_board.uncolored(),
                en_passant_capture_square,
                hash: zobrist::piece_table(&piece_board)
                    ^ zobrist::side(current_color)
                    ^ en_passant_capture_square
                        .map_or(0, |square| zobrist::en_passant_file(square.file()))
                    ^ zobrist::castling_rights(&white.castling_rights)
                    ^ zobrist::castling_rights(&black.castling_rights),
                checkers: BitBoard::EMPTY,
                pinned: BitBoard::EMPTY,
                min_ply_clock: ply_clock,
                full_moves,
            };

            board.update_move_restrictions();

            let is_impossible_en_passant_square = en_passant_capture_square.is_some_and(|square| {
                let is_impossible_capture_square = (BitBoard::from(square)
                    & match current_color {
                        Color::White => BitBoard::WHITE_EN_PASSANT_CAPTURE_RANKS,
                        Color::Black => BitBoard::BLACK_EN_PASSANT_CAPTURE_RANKS,
                    })
                .is_empty();

                let with_mismatching_pawn = piece_board.pieces()
                    [unsafe { square.move_one_down_unchecked(current_color) }]
                    != Some(Piece {
                        kind: PieceKind::Pawn,
                        color: !current_color,
                    });

                is_impossible_capture_square || with_mismatching_pawn
            });

            let mut enemy_king_checkers = BitBoard::EMPTY;

            let king_square = unsafe { Square::try_from(board.them.king).unwrap_unchecked() };

            enemy_king_checkers ^= index::knight_attacks(king_square) & board.us.knights;
            enemy_king_checkers ^=
                index::pawn_attacks(king_square, !current_color) & board.us.pawns;

            // Get all the sliding pieces that could be attacking the enemy king
            let attackers = (index::rook_slides(king_square, board.us.occupation)
                & (board.us.rooks + board.us.queens))
                + (index::bishop_slides(king_square, board.us.occupation)
                    & (board.us.bishops + board.us.queens));

            // Update pins
            for attacker in attackers.bits() {
                let pieces_between =
                    index::line_between(attacker, king_square) & board.them.occupation;

                if pieces_between.is_empty() {
                    enemy_king_checkers ^= attacker.into();
                }
            }

            if !board.us.king.is_a_single_one() || !board.them.king.is_a_single_one() {
                Err(ParseBoardError::InvalidKingCount)
            } else if !(board.us.pawns & BitBoard::EDGE_RANKS).is_empty() {
                Err(ParseBoardError::PawnsOnEdgeRanks)
            } else if is_impossible_en_passant_square {
                Err(ParseBoardError::InvalidEnPassantSquare(None))
            } else if !enemy_king_checkers.is_empty() {
                Err(ParseBoardError::CapturableKing)
            } else {
                Ok(board)
            }
        }
    }
}

impl Display for Board {
    // TODO: Refactor this to look nicer
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        {
            for column in (0..8).rev() {
                let mut spacing = 0;

                for row in 0..8 {
                    let square = Square::try_from(column * 8 + row).unwrap();
                    let piece = self.piece(square);

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

        self.playing_color.fmt(f)?;

        ' '.fmt(f)?;

        let (white, black) = match self.playing_color {
            Color::White => (self.us, self.them),
            Color::Black => (self.them, self.us),
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

        if let Some(square) = self.en_passant_capture_square {
            square.fmt(f)?;
        } else {
            '-'.fmt(f)?;
        }
        f.write_fmt(format_args!(" {} {}", self.min_ply_clock, self.full_moves))
    }
}

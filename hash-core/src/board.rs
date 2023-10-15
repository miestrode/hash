use std::{fmt, fmt::Display, mem, str::FromStr};

use crate::{
    cache::CacheHash,
    index,
    index::zobrist,
    mg,
    repr::{ColoredPieceTable, Move, Piece, PieceKind, PieceTable, Player},
};
use hash_bootstrap::{BitBoard, Color, Square};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Board {
    pub us: Player,
    pub them: Player,
    pub checkers: BitBoard,
    pub pinned: BitBoard,
    pub playing_color: Color,
    pub en_passant_capture_square: Option<Square>,
    pub piece_table: PieceTable,
    pub min_ply_clock: u8,
    pub full_moves: u16,
    pub hash: u64,
}

impl CacheHash for Board {
    fn hash(&self) -> u64 {
        self.hash
    }
}

impl Board {
    pub fn starting_position() -> Self {
        // Taken from https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation
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

        // Get all the sliding pieces that could be attacking the enemy king
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
    pub unsafe fn make_move_unchecked(&mut self, chess_move: &Move) {
        self.en_passant_capture_square = None;
        self.checkers = BitBoard::EMPTY;
        self.pinned = BitBoard::EMPTY;

        // SAFETY: The board is assumed to be valid
        let enemy_king_square = unsafe { Square::try_from(self.them.king).unwrap_unchecked() };
        let moved_piece_kind = self.piece_table.piece_kind(chess_move.origin).unwrap();
        let target_piece_kind = self.piece_table.piece_kind(chess_move.target);

        let mut is_capture = false;

        self.us.castling_rights.revoke(chess_move.origin);
        self.them.castling_rights.revoke(chess_move.target);

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
                    // If we are here, this must mean the move was an en-passant.
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

    pub fn gen_child_boards(&self) -> impl Iterator<Item = (Move, Board)> + '_ {
        mg::gen_moves(self).into_iter().map(|chess_move| {
            let mut new_board = *self;
            unsafe {
                new_board.make_move_unchecked(&chess_move);
            };

            (chess_move, new_board)
        })
    }
}

impl FromStr for Board {
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
                square => Some(Square::from_str(square)?),
            };

            let ply_clock = parts[4]
                .parse::<u8>()
                .map_err(|_| "Input contains invalid number for the half-move clock")?;

            let full_moves = parts[5]
                .parse::<u16>()
                .map_err(|_| "Input contains invalid number for full-moves")?;

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

            let mut board = Board {
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
                min_ply_clock: ply_clock,
                full_moves,
            };

            board.update_move_restrictions();

            Ok(board)
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

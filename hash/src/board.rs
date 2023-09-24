use std::mem;

use hash_bootstrap::{BitBoard, Color, Square};

use crate::{
    cache::CacheHash,
    index, mg,
    repr::{Move, Piece, PieceKind, PieceTable, Player},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Board {
    pub(crate) us: Player,
    pub(crate) them: Player,
    pub(crate) checkers: BitBoard,
    pub(crate) pinned: BitBoard,
    pub(crate) playing_color: Color,
    pub(crate) en_passant_capture_square: Option<Square>,
    pub(crate) piece_table: PieceTable,
    pub(crate) hash: u64,
}

impl CacheHash for Board {
    fn hash(&self) -> u64 {
        self.hash
    }
}

impl Board {
    pub(crate) fn is_attacked(&self, square: Square) -> bool {
        let mut attackers = BitBoard::EMPTY;
        let occupation = self.occupation() - self.us.king;

        attackers +=
            index::rook_slides(square, occupation) & (self.them.rooks + self.them.queens);
        attackers += index::bishop_slides(square, occupation)
            & (self.them.bishops + self.them.queens);

        attackers += index::knight_attacks(square) & self.them.knights;
        attackers += index::king_attacks(square) & self.them.king;

        let square: BitBoard = square.into();

        attackers += (square.move_one_up_left(self.playing_color)
            + square.move_one_up_right(self.playing_color))
            & self.them.pawns;

        !attackers.is_empty()
    }

    pub(crate) fn in_check(&self) -> bool {
        !self.checkers.is_empty()
    }

    pub(crate) fn occupation(&self) -> BitBoard {
        self.us.occupation + self.them.occupation
    }

    pub(crate) fn piece(&self, square: Square) -> Option<Piece> {
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

    pub(crate) fn update_move_restrictions(&mut self) {
        let king_square = self.us.king.try_into().unwrap();

        self.checkers ^= index::knight_attacks(king_square) & self.them.knights;
        self.checkers ^= index::pawn_attacks(king_square, self.playing_color) & self.them.pawns;

        // Get all the sliding pieces that could be attacking the enemy king
        let attackers = (index::rook_slides(king_square, self.them.occupation)
            & (self.them.rooks + self.them.queens))
            + (index::bishop_slides(king_square, self.them.occupation)
            & (self.them.bishops + self.them.queens));

        // Update pins
        for attacker in attackers.bits() {
            let pieces_between =
                index::line_between(attacker, king_square) & self.us.occupation;

            if pieces_between.is_empty() {
                self.checkers ^= attacker.into();
            } else if pieces_between.is_a_single_one() {
                self.pinned ^= pieces_between;
            }
        }
    }

    // INVARIANT: The passed move must be legal in relation to the current board.
    // NOTE: The method returns true if the move was a pawn move or a capture
    pub(crate) unsafe fn make_move_unchecked(&mut self, chess_move: &Move) -> bool {
        self.en_passant_capture_square = None;
        self.checkers = BitBoard::EMPTY;
        self.pinned = BitBoard::EMPTY;

        let enemy_king_square: Square = self.them.king.try_into().unwrap();
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
                if chess_move.origin.rank().abs_diff(chess_move.target.rank()) == 2
                {
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
                    origin.try_into().unwrap(),
                    Piece {
                        kind: PieceKind::Rook,
                        color: self.playing_color,
                    },
                );

                self.add_piece_unchecked(
                    target.try_into().unwrap(),
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

        self.playing_color = !self.playing_color;

        moved_piece_kind == PieceKind::Pawn || is_capture
    }

    pub(crate) fn perft(&self, depth: u32) -> u64 {
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
}

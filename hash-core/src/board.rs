use std::{mem, str::FromStr};

use hash_bootstrap::{BitBoard, Color, Square};

use crate::index::zobrist;
use crate::mg::{Bishop, Gen, King, Knight, Rook};
use crate::{
    cache::CacheHash,
    index, mg,
    repr::{EnPassantData, Move, MoveMetadata, Piece, PieceKind, PieceTable, Player},
};

#[derive(Clone, Copy)]
pub(crate) struct Board {
    pub(crate) current_player: Player,
    pub(crate) opposing_player: Player,
    pub(crate) checkers: BitBoard,
    pub(crate) pinned: BitBoard,
    pub(crate) current_color: Color,
    piece_table: PieceTable,
    en_passant_data: Option<EnPassantData>,
    hash: u64,
}

impl CacheHash for Board {
    fn hash(&self) -> u64 {
        self.hash
    }
}

impl Board {
    pub(crate) fn is_attacked(&self, square: Square) -> bool {
        let mut attackers = BitBoard::EMPTY;

        attackers += Rook::pseudo_legal_moves(
            square,
            BitBoard::EMPTY,
            self.occupation(),
            self.current_color,
        ) & (self.opposing_player.rooks + self.opposing_player.queens);

        attackers += Bishop::pseudo_legal_moves(
            square,
            BitBoard::EMPTY,
            self.occupation(),
            self.current_color,
        ) & (self.opposing_player.bishops + self.opposing_player.queens);

        attackers += Knight::pseudo_legal_moves(
            square,
            BitBoard::EMPTY,
            BitBoard::EMPTY,
            self.current_color,
        ) & self.opposing_player.knights;

        attackers +=
            King::pseudo_legal_moves(square, BitBoard::EMPTY, BitBoard::EMPTY, self.current_color)
                & self.opposing_player.king;

        let square: BitBoard = square.into();

        attackers |= (square.move_one_up_left(self.current_color)
            + square.move_one_up_right(self.current_color))
            & self.opposing_player.pawns;

        return !attackers.is_empty();
    }

    pub(crate) fn in_check(&self) -> bool {
        !self.checkers.is_empty()
    }

    pub(crate) fn occupation(&self) -> BitBoard {
        self.current_player.occupation + self.opposing_player.occupation
    }

    pub fn white_player(&self) -> &Player {
        match self.current_color {
            Color::White => &self.current_player,
            Color::Black => &self.opposing_player,
        }
    }

    pub fn black_player(&self) -> &Player {
        match self.current_color {
            Color::White => &self.opposing_player,
            Color::Black => &self.current_player,
        }
    }

    pub fn get_piece(&self, square: Square) -> Option<Piece> {
        self.piece_table.0[square].map(|kind| Piece {
            kind,
            color: if self.current_player.occupation.get_bit(square) {
                self.current_color
            } else {
                !self.current_color
            },
        })
    }

    pub(crate) unsafe fn move_piece_unchecked(
        &mut self,
        kind: PieceKind,
        origin: Square,
        target: Square,
    ) {
        let captured_kind = self.piece_table.piece_kind(target);
        self.piece_table.move_piece(origin, target);

        self.hash ^= zobrist::piece(
            Piece {
                kind,
                color: self.current_color,
            },
            origin,
        ) ^ zobrist::piece(
            Piece {
                kind,
                color: self.current_color,
            },
            target,
        );

        // SAFETY: Data is assumed to be valid
        unsafe {
            self.current_player
                .move_piece_unchecked(kind, origin, target);
            if let Some(captured_kind) = captured_kind {
                self.hash ^= zobrist::piece(
                    Piece {
                        kind: captured_kind,
                        color: !self.current_color,
                    },
                    target,
                );

                self.opposing_player.toggle_piece(captured_kind, target);
            }
        }
    }

    // SAFETY: This function assumes the move at hand is actually properly constructed and legal
    // NOTE: The function returns a boolean representing whether the move was a pawn move or piece
    // capture
    pub unsafe fn make_move_unchecked(&mut self, chess_move: &Move) -> bool {
        let past_ep_data = self.en_passant_data;
        self.en_passant_data = None;

        if let Some(ep_data) = past_ep_data {
            self.hash ^= zobrist::en_passant_file(ep_data.pawn.file());
        }

        // Remove the previous castling rights, "stored" in the hash
        self.hash ^= zobrist::castling_rights(&self.current_player.castling_rights)
            ^ zobrist::castling_rights(&self.opposing_player.castling_rights);

        // This only actually affects things if the piece moved captured a castling piece or was a
        // castling piece
        self.current_player.castling_rights.0[chess_move.origin] = false;
        self.opposing_player.castling_rights.0[chess_move.target] = false;

        // Add the new castling rights
        self.hash ^= zobrist::castling_rights(&self.current_player.castling_rights)
            ^ zobrist::castling_rights(&self.opposing_player.castling_rights);

        let is_capture = self.piece_table.0[chess_move.target].is_some();
        let is_pawn_move = chess_move.piece_kind == PieceKind::Pawn;

        // SAFETY: See above
        // TODO: Check if indexing into the piece table like this is faster than storing this
        // information on the move.
        unsafe {
            self.move_piece_unchecked(chess_move.piece_kind, chess_move.origin, chess_move.target)
        };

        match chess_move.metadata {
            MoveMetadata::Promotion(kind) => {
                self.piece_table.set(Some(kind), chess_move.target);
                self.current_player
                    .toggle_piece(PieceKind::Pawn, chess_move.target);
                self.current_player.toggle_piece(kind, chess_move.target);

                // TODO: Check if there is any optimization to be had by making the hash changes
                // manual (and not be done automatically by the "move_piece_unchecked" function)
                self.hash ^= zobrist::piece(
                    Piece {
                        kind: PieceKind::Pawn,
                        color: self.current_color,
                    },
                    chess_move.target,
                ) ^ zobrist::piece(
                    Piece {
                        kind,
                        color: self.current_color,
                    },
                    chess_move.target,
                )
            }
            MoveMetadata::EnPassant => {
                // SAFETY: See above
                let pawn_square = past_ep_data.unwrap().pawn;
                self.opposing_player
                    .toggle_piece(PieceKind::Pawn, pawn_square);
                self.piece_table.set(None, pawn_square);

                self.hash ^= zobrist::piece(
                    Piece {
                        kind: PieceKind::Pawn,
                        color: !self.current_color,
                    },
                    pawn_square,
                );
            }
            MoveMetadata::DoublePush => {
                self.hash ^= zobrist::en_passant_file(chess_move.origin.file());

                self.en_passant_data = Some(EnPassantData {
                    // SAFETY: See above.
                    capture_point: unsafe {
                        chess_move
                            .target
                            .move_one_down_unchecked(self.current_color)
                    }
                    .as_bitboard(),
                    pawn: chess_move.target,
                });
            }
            MoveMetadata::CastleKingSide => {
                // Based on https://en.wikipedia.org/wiki/Castling
                let (initial_rook, end_rook) = match self.current_color {
                    Color::White => (Square::BOTTOM_RIGHT_ROOK, Square::F1),
                    Color::Black => (Square::TOP_RIGHT_ROOK, Square::F8),
                };

                // SAFETY: See above
                // TODO: Consider using a specialized function to avoid the capture checks that are
                // irrelevant if performance is improved
                unsafe {
                    self.move_piece_unchecked(PieceKind::Rook, initial_rook, end_rook);
                }
            }
            MoveMetadata::CastleQueenSide => {
                // Based on https://en.wikipedia.org/wiki/Castling
                let (initial_rook, end_rook) = match self.current_color {
                    Color::White => (Square::BOTTOM_LEFT_ROOK, Square::D1),
                    Color::Black => (Square::TOP_LEFT_ROOK, Square::D8),
                };

                // SAFETY: See above
                // TODO: Consider using a specialized function to avoid the capture checks that are
                // irrelevant if performance is improved
                unsafe {
                    self.move_piece_unchecked(PieceKind::Rook, initial_rook, end_rook);
                }
            }
            MoveMetadata::None => {}
        }

        self.hash ^= zobrist::side(self.current_color) ^ zobrist::side(!self.current_color);
        self.current_color = !self.current_color;

        mem::swap(&mut self.current_player, &mut self.opposing_player);

        is_pawn_move || is_capture
    }
}

use arrayvec::ArrayVec;
use hash_bootstrap::{BitBoard, Color, Square};

use crate::{
    board::Board,
    index,
    repr::{ChessMove, PieceKind},
};

/// The maximum number of moves stored by [`Moves`]. This shouldn't be relevant for most
/// cases - simply use [`Moves`].
pub const MOVES: usize = 218;

/// An array of moves that is the output of move generation ([`mg::gen_moves`]).
pub type Moves = ArrayVec<ChessMove, MOVES>;

trait CheckType {
    const IN_CHECK: bool;
}

struct InCheck;

struct NotInCheck;

impl CheckType for InCheck {
    const IN_CHECK: bool = true;
}

impl CheckType for NotInCheck {
    const IN_CHECK: bool = false;
}

trait Gen {
    const PIECE_KIND: PieceKind;

    fn pseudo_legal_moves(
        origin: Square,
        friendly_occupation: BitBoard,
        occupation: BitBoard,
        color: Color,
    ) -> BitBoard;

    fn legal_moves<C: CheckType>(board: &Board, moves: &mut Moves) {
        let pieces = board.us.piece_bitboard(Self::PIECE_KIND);
        let occupation = board.occupation();

        // SAFETY: The board is assumed to be validly constructed
        let king_square = unsafe { Square::try_from(board.us.king).unwrap_unchecked() };

        let valid_targets = if C::IN_CHECK {
            // NOTE: If this code was invoked there is a single checker. If it isn't a sliding piece
            // there won't be a line between the checker and the king (except for pawns, but in this
            // case the line is fine).
            // Likewise we are adding the checker bitboard, as the line between the squares
            // doesn't include its edge points and for the cases where we don't have sliding piece
            // checking the king.
            // SAFETY: The board is assumed to be validly constructed
            board.checkers
                ^ index::line_between(
                    unsafe { Square::try_from(board.checkers).unwrap_unchecked() },
                    king_square,
                )
        } else {
            BitBoard::FULL
        };

        moves.extend((pieces & !board.pinned).bits().flat_map(|piece| {
            (Self::pseudo_legal_moves(piece, board.us.occupation, occupation, board.playing_color)
                & valid_targets)
                .bits()
                .map(move |target| ChessMove {
                    origin: piece,
                    target,
                    promotion: None,
                })
        }));

        if !C::IN_CHECK {
            moves.extend((pieces & board.pinned).bits().flat_map(|piece| {
                (Self::pseudo_legal_moves(
                    piece,
                    board.us.occupation,
                    occupation,
                    board.playing_color,
                ) & index::line_fit(king_square, piece))
                .bits()
                .map(move |target| ChessMove {
                    origin: piece,
                    target,
                    promotion: None,
                })
            }));
        }
    }
}

pub struct Pawn;

impl Pawn {
    unsafe fn is_legal_en_passant_capture(
        board: &Board,
        en_passant_capture_square: Square,
        origin: Square,
    ) -> bool {
        let mut occupation = board.occupation();

        // Update board to it's post capture state
        occupation.toggle_bit(origin);

        // SAFETY: Data passed and board are assumed to be valid
        occupation.toggle_bit(unsafe {
            en_passant_capture_square.move_one_down_unchecked(board.playing_color)
        });
        occupation.toggle_bit(en_passant_capture_square);

        // SAFETY: Data passed and board are assumed to be valid
        let king_square = unsafe { Square::try_from(board.us.king).unwrap_unchecked() };

        // Test for any rays hitting the king
        ((index::bishop_slides(king_square, occupation) & (board.them.queens | board.them.bishops))
            | (index::rook_slides(king_square, occupation)
                & (board.them.queens | board.them.rooks)))
            .is_empty()
    }
}

impl Gen for Pawn {
    const PIECE_KIND: PieceKind = PieceKind::Pawn;

    fn pseudo_legal_moves(
        origin: Square,
        friendly_occupation: BitBoard,
        occupation: BitBoard,
        color: Color,
    ) -> BitBoard {
        index::pawn_moves(
            origin,
            friendly_occupation & !BitBoard::from(origin),
            occupation & !friendly_occupation,
            color,
        )
    }

    fn legal_moves<C: CheckType>(board: &Board, moves: &mut Moves) {
        let occupation = board.occupation();

        // SAFETY: The board is assumed to be validly constructed
        let king_square = unsafe { Square::try_from(board.us.king).unwrap_unchecked() };

        let valid_targets = if C::IN_CHECK {
            // NOTE: If this code was invoked there is a single checker. If it isn't a sliding piece
            // there won't be a line between the checker and the king (except for pawns, but in this
            // case the line is fine).
            // Likewise we are adding the checker bitboard, as the line between the squares
            // doesn't include its edge points and for the cases where we don't have sliding piece
            // checking the king.
            // SAFETY: The board is assumed to be validly constructed
            board.checkers
                ^ index::line_between(
                    unsafe { Square::try_from(board.checkers).unwrap_unchecked() },
                    king_square,
                )
        } else {
            BitBoard::FULL
        };

        moves.extend((board.us.pawns & !board.pinned).bits().flat_map(|piece| {
            ((Self::pseudo_legal_moves(
                piece,
                board.us.occupation,
                occupation,
                board.playing_color,
            ) & valid_targets)
                & !BitBoard::EDGE_RANKS)
                .bits()
                .map(move |target| ChessMove {
                    origin: piece,
                    target,
                    promotion: None,
                })
        }));

        // Promotions
        moves.extend((board.us.pawns & !board.pinned).bits().flat_map(|piece| {
            (Self::pseudo_legal_moves(piece, board.us.occupation, occupation, board.playing_color)
                & valid_targets
                & BitBoard::EDGE_RANKS)
                .bits()
                .flat_map(move |target| {
                    PieceKind::PROMOTIONS
                        .into_iter()
                        .map(move |kind| ChessMove {
                            origin: piece,
                            target,
                            promotion: Some(kind),
                        })
                })
        }));

        if !C::IN_CHECK {
            moves.extend((board.us.pawns & board.pinned).bits().flat_map(|piece| {
                ((Self::pseudo_legal_moves(
                    piece,
                    board.us.occupation,
                    occupation,
                    board.playing_color,
                ) & index::line_fit(king_square, piece))
                    & !BitBoard::EDGE_RANKS)
                    .bits()
                    .map(move |target| ChessMove {
                        origin: piece,
                        target,
                        promotion: None,
                    })
            }));

            // Promotions
            moves.extend((board.us.pawns & board.pinned).bits().flat_map(|piece| {
                (Self::pseudo_legal_moves(
                    piece,
                    board.us.occupation,
                    occupation,
                    board.playing_color,
                ) & index::line_fit(king_square, piece)
                    & BitBoard::EDGE_RANKS)
                    .bits()
                    .flat_map(move |target| {
                        PieceKind::PROMOTIONS
                            .into_iter()
                            .map(move |kind| ChessMove {
                                origin: piece,
                                target,
                                promotion: Some(kind),
                            })
                    })
            }));
        }

        // En passants
        unsafe {
            if let Some(en_passant_capture_square) = board.en_passant_capture_square {
                for origin in
                    index::pawn_attacks(en_passant_capture_square, !board.playing_color).bits()
                {
                    if board.us.pawns.get_bit(origin)
                        && Pawn::is_legal_en_passant_capture(
                            board,
                            en_passant_capture_square,
                            origin,
                        )
                    {
                        moves.push(ChessMove {
                            origin,
                            target: en_passant_capture_square,
                            promotion: None,
                        });
                    }
                }
            }
        }
    }
}

pub struct Knight;

impl Gen for Knight {
    const PIECE_KIND: PieceKind = PieceKind::Knight;

    fn pseudo_legal_moves(
        origin: Square,
        friendly_occupation: BitBoard,
        _occupation: BitBoard,
        _color: Color,
    ) -> BitBoard {
        index::knight_attacks(origin) & !friendly_occupation
    }

    // This is essentially identical to the regular `legal_moves`, except we don't care about pinned
    // pieces, as a pinned knight cannot move.

    fn legal_moves<C: CheckType>(board: &Board, moves: &mut Moves) {
        let occupation = board.occupation();

        // SAFETY: The board is assumed to be validly constructed
        let king_square = unsafe { Square::try_from(board.us.king).unwrap_unchecked() };

        let valid_targets = if C::IN_CHECK {
            // NOTE: If this code was invoked there is a single checker. If it isn't a sliding piece
            // there won't be a line between the checker and the king (except for pawns, but in this
            // case the line is fine).
            // Likewise we are adding the checker bitboard, as the line between the squares
            // doesn't include its edge points and for the cases where we don't have sliding piece
            // SAFETY: The board is assumed to be validly constructed
            board.checkers
                ^ index::line_between(
                    unsafe { Square::try_from(board.checkers).unwrap_unchecked() },
                    king_square,
                )
        } else {
            BitBoard::FULL
        };

        moves.extend((board.us.knights & !board.pinned).bits().flat_map(|piece| {
            (Self::pseudo_legal_moves(piece, board.us.occupation, occupation, board.playing_color)
                & valid_targets)
                .bits()
                .map(move |target| ChessMove {
                    origin: piece,
                    target,
                    promotion: None,
                })
        }));
    }
}

pub struct Bishop;

impl Gen for Bishop {
    const PIECE_KIND: PieceKind = PieceKind::Bishop;

    fn pseudo_legal_moves(
        origin: Square,
        friendly_occupation: BitBoard,
        occupation: BitBoard,
        _color: Color,
    ) -> BitBoard {
        index::bishop_slides(origin, occupation) & !friendly_occupation
    }
}

pub struct Rook;

impl Gen for Rook {
    const PIECE_KIND: PieceKind = PieceKind::Rook;

    fn pseudo_legal_moves(
        origin: Square,
        friendly_occupation: BitBoard,
        occupation: BitBoard,
        _color: Color,
    ) -> BitBoard {
        index::rook_slides(origin, occupation) & !friendly_occupation
    }
}

pub struct Queen;

impl Gen for Queen {
    const PIECE_KIND: PieceKind = PieceKind::Queen;

    fn pseudo_legal_moves(
        origin: Square,
        friendly_occupation: BitBoard,
        occupation: BitBoard,
        _color: Color,
    ) -> BitBoard {
        (index::rook_slides(origin, occupation) | index::bishop_slides(origin, occupation))
            & !friendly_occupation
    }
}

pub struct King;

impl Gen for King {
    const PIECE_KIND: PieceKind = PieceKind::King;

    fn pseudo_legal_moves(
        origin: Square,
        friendly_occupation: BitBoard,
        _occupation: BitBoard,
        _color: Color,
    ) -> BitBoard {
        index::king_attacks(origin) & !friendly_occupation
    }

    fn legal_moves<C: CheckType>(board: &Board, moves: &mut Moves) {
        // SAFETY: The board is assumed to be validly constructed
        let king_square = unsafe { Square::try_from(board.us.king).unwrap_unchecked() };

        moves.extend(
            Self::pseudo_legal_moves(
                king_square,
                board.us.occupation,
                board.occupation(),
                board.playing_color,
            )
            .bits()
            .filter(|square| !board.is_attacked_by_them(*square))
            .map(|target| ChessMove {
                origin: king_square,
                target,
                promotion: None,
            }),
        );

        // Castles
        if !C::IN_CHECK {
            let king_side_castle_mask = BitBoard::king_side_castle_mask(board.playing_color);

            if board.us.castling_rights.can_castle_king_side()
                && (king_side_castle_mask & board.occupation()).is_empty()
                && (king_side_castle_mask
                    .bits()
                    .all(|square| !board.is_attacked_by_them(square)))
            {
                moves.push(ChessMove {
                    origin: king_square,
                    target: match board.playing_color {
                        Color::White => Square::G1,
                        Color::Black => Square::G8,
                    },
                    promotion: None,
                });
            }

            if board.us.castling_rights.can_castle_queen_side()
                && (BitBoard::queen_side_castle_occupation_mask(board.playing_color)
                    & board.occupation())
                .is_empty()
                && (BitBoard::queen_side_castle_attack_mask(board.playing_color)
                    .bits()
                    .all(|square| !board.is_attacked_by_them(square)))
            {
                moves.push(ChessMove {
                    origin: king_square,
                    target: match board.playing_color {
                        Color::White => Square::C1,
                        Color::Black => Square::C8,
                    },
                    promotion: None,
                });
            }
        }
    }
}

pub fn gen_moves(board: &Board) -> Moves {
    let mut moves = Moves::new();

    if board.in_check() {
        King::legal_moves::<InCheck>(board, &mut moves);

        if board.checkers.count_ones() < 2 {
            Pawn::legal_moves::<InCheck>(board, &mut moves);
            Knight::legal_moves::<InCheck>(board, &mut moves);
            Bishop::legal_moves::<InCheck>(board, &mut moves);
            Rook::legal_moves::<InCheck>(board, &mut moves);
            Queen::legal_moves::<InCheck>(board, &mut moves);
        }
    } else {
        King::legal_moves::<NotInCheck>(board, &mut moves);
        Pawn::legal_moves::<NotInCheck>(board, &mut moves);
        Knight::legal_moves::<NotInCheck>(board, &mut moves);
        Bishop::legal_moves::<NotInCheck>(board, &mut moves);
        Rook::legal_moves::<NotInCheck>(board, &mut moves);
        Queen::legal_moves::<NotInCheck>(board, &mut moves);
    }

    moves
}

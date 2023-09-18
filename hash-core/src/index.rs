#[cfg(target_feature = "bmi2")]
use std::arch::x86_64;

use hash_bootstrap::{BitBoard, Color, Square};

#[derive(Clone, Copy)]
struct Metadata {
    offset: usize,
    mask: BitBoard,
    #[cfg(not(target_feature = "bmi2"))]
    pub magic: u64,
}

rustifact::use_symbols!(
    CROSS_SLIDES,
    CROSS_META,
    DIAGONAL_SLIDES,
    DIAGONAL_META,
    KNIGHT_ATTACKS,
    KING_ATTACKS,
    LINE,
    BETWEEN,
    WHITE_PAWN_ATTACKS,
    BLACK_PAWN_ATTACKS,
    WHITE_PAWN_PUSHES,
    BLACK_PAWN_PUSHES,
    ZOBRIST_MAP
);

#[cfg(target_feature = "bmi2")]
/// Returns the bitboard of every square a rook can reach when on the passed `origin` square.
/// The `blockers` bitboard allows one to restrict the rooks movement, as a rook cannot jump over
/// a "blocker" (although it can eat it).
///
/// This function may be implemented using PEXT on systems supporting this feature and will
/// otherwise use magic bitboards.
///
/// # Example
/// Given a rook on D4, and a set of blockers:
/// ```text
/// . 1 . . . . . .
/// . . . . 1 . . .
/// . . . 1 . . . .
/// . . . . . . . .
/// . . . X . . 1 .
/// . . . . . . . .
/// 1 . . . . 1 . .
/// . . . 1 . . . .
/// ```
///
/// The result would be:
/// ```text
/// . . . . . . . .
/// . . . . . . . .
/// . . . 1 . . . .
/// . . . 1 . . . .
/// 1 1 1 X 1 1 1 .
/// . . . 1 . . . .
/// . . . 1 . . . .
/// . . . 1 . . . .
/// ```
///
/// Where the square marked with an `X` is where our rook is. Notice how the final output includes
/// the squares of the blockers reachable by the rook. Likewise note how the blockers on the edges
/// of the board didn't make any difference to the output.
pub(crate) fn rook_slides(origin: Square, blockers: BitBoard) -> BitBoard {
    let metadata = CROSS_META[origin];

    *unsafe {
        CROSS_SLIDES.get_unchecked(
            metadata.offset + x86_64::_pext_u64(blockers.0, metadata.mask.0) as usize,
        )
    }
}

#[cfg(not(target_feature = "bmi2"))]
/// Returns the bitboard of every square a rook can reach when on the passed `origin` square.
/// The `blockers` bitboard allows one to restrict the rooks movement, as a rook cannot jump over
/// a "blocker" (although it can eat it).
///
/// This function may be implemented using PEXT on systems supporting this feature and will
/// otherwise use magic bitboards.
///
/// # Example
/// Given a rook on D4, and a set of blockers:
/// ```text
/// . 1 . . . . . .
/// . . . . 1 . . .
/// . . . 1 . . . .
/// . . . . . . . .
/// . . . X . . 1 .
/// . . . . . . . .
/// 1 . . . . 1 . .
/// . . . 1 . . . .
/// ```
///
/// The result would be:
/// ```text
/// . . . . . . . .
/// . . . . . . . .
/// . . . 1 . . . .
/// . . . 1 . . . .
/// 1 1 1 X 1 1 1 .
/// . . . 1 . . . .
/// . . . 1 . . . .
/// . . . 1 . . . .
/// ```
///
/// Where the square marked with an `X` is where our rook is. Notice how the final output includes
/// the squares of the blockers reachable by the rook. Likewise note how the blockers on the edges
/// of the board didn't make any difference to the output.
pub(crate) fn rook_slides(origin: Square, blockers: BitBoard) -> BitBoard {
    let metadata = CROSS_META[origin];
    unsafe {
        *SLIDES.get_unchecked(
            metadata.offset
                + ((blockers & metadata.mask).0.wrapping_mul(metadata.magic) >> (64 - 12)) as usize,
        )
    }
}

#[cfg(target_feature = "bmi2")]
/// Returns the bitboard of every square a rook can reach when on the passed `origin` square.
/// The `blockers` bitboard allows one to restrict the rook's movement, as a bishop cannot jump over
/// a "blocker" (although it can eat it).
///
/// This function may be implemented using PEXT on systems supporting this feature and will
/// otherwise use magic bitboards.
///
/// # Example
/// Given a bishop on D4, and a set of blockers:
/// ```text
/// . 1 . . . . . 1
/// . 1 . . 1 . . .
/// . . . 1 . . . .
/// . . . . . . . .
/// . . . X . . 1 .
/// . . . . . . . .
/// 1 1 . . . 1 . .
/// . . . 1 . . . .
/// ```
///
/// The result would be:
/// ```text
/// . . . . . . . 1
/// 1 . . . . . 1 .
/// . 1 . . . 1 . .
/// . . 1 . 1 . . .
/// . . . X . . . .
/// . . 1 . 1 . . .
/// . 1 . . . 1 . .
/// . . . . . . . .
/// ```
///
/// Where the square marked with an `X` is where our bishop is. Notice how the final output includes
/// the squares of the blockers reachable by the bishop. Likewise note how the blockers on the edges
/// of the board didn't make any difference to the output.
pub(crate) fn bishop_slides(origin: Square, blockers: BitBoard) -> BitBoard {
    let metadata = DIAGONAL_META[origin];

    *unsafe {
        DIAGONAL_SLIDES.get_unchecked(
            metadata.offset + x86_64::_pext_u64(blockers.0, metadata.mask.0) as usize,
        )
    }
}

#[cfg(not(target_feature = "bmi2"))]
/// Returns the bitboard of every square a rook can reach when on the passed `origin` square.
/// The `blockers` bitboard allows one to restrict the rook's movement, as a bishop cannot jump over
/// a "blocker" (although it can eat it).
///
/// This function may be implemented using PEXT on systems supporting this feature and will
/// otherwise use magic bitboards.
///
/// # Example
/// Given a bishop on D4, and a set of blockers:
/// ```text
/// . 1 . . . . . 1
/// . 1 . . 1 . . .
/// . . . 1 . . . .
/// . . . . . . . .
/// . . . X . . 1 .
/// . . . . . . . .
/// 1 1 . . . 1 . .
/// . . . 1 . . . .
/// ```
///
/// The result would be:
/// ```text
/// . . . . . . . 1
/// 1 . . . . . 1 .
/// . 1 . . . 1 . .
/// . . 1 . 1 . . .
/// . . . X . . . .
/// . . 1 . 1 . . .
/// . 1 . . . 1 . .
/// . . . . . . . .
/// ```
///
/// Where the square marked with an `X` is where our bishop is. Notice how the final output includes
/// the squares of the blockers reachable by the bishop. Likewise note how the blockers on the edges
/// of the board didn't make any difference to the output.
pub(crate) fn bishop_slides(origin: Square, blockers: BitBoard) -> BitBoard {
    let metadata = DIAGONAL_META[origin];
    unsafe {
        *SLIDES.get_unchecked(
            metadata.offset
                + ((blockers & metadata.mask).0.wrapping_mul(metadata.magic) >> (64 - 9)) as usize,
        )
    }
}

/// Returns a bitboard of all squares that a knight could move to if on the passed square
/// (hypothetically).
///
/// # Example
/// ```ignore
/// assert_eq!(index::knight_attacks(Square::D4), bb!(
///     0b00000000
///     0b00000000
///     0b00101000
///     0b01000100
///     0b00000000
///     0b01000100
///     0b00101000
///     0b00000000
/// ))
/// ```
pub(crate) fn knight_attacks(origin: Square) -> BitBoard {
    KNIGHT_ATTACKS[origin]
}

/// Returns a bitboard of all squares that a king could move to if on the passed square
/// (hypothetically).
///
/// # Example
/// ```ignore
/// assert_eq!(index::king_attacks(Square::E1), bb!(
///     0b00000000
///     0b00000000
///     0b00000000
///     0b00000000
///     0b00000000
///     0b00000000
///     0b00011100
///     0b00010100
/// ))
/// ```
pub(crate) fn king_attacks(origin: Square) -> BitBoard {
    KING_ATTACKS[origin]
}

/// Returns a bitboard of all squares that a pawn could attack if on the passed square.
pub(crate) fn pawn_attacks(origin: Square, color: Color) -> BitBoard {
    match color {
        Color::White => WHITE_PAWN_ATTACKS[origin],
        Color::Black => BLACK_PAWN_ATTACKS[origin],
    }
}

/// Returns a bitboard of all squares that a pawn could move to if on the passed square, given
/// a set of friendly blockers and blockers, and the color to do this in relation to.
pub(crate) fn pawn_moves(
    origin: Square,
    friendly_blockers: BitBoard,
    enemy_blockers: BitBoard,
    color: Color,
) -> BitBoard {
    let blockers = friendly_blockers + enemy_blockers;
    let smear = blockers.smear_one_up(color);

    match color {
        Color::White => {
            (WHITE_PAWN_ATTACKS[origin] & enemy_blockers) + WHITE_PAWN_PUSHES[origin] - smear
        }
        Color::Black => {
            (BLACK_PAWN_ATTACKS[origin] & enemy_blockers) + BLACK_PAWN_PUSHES[origin] - smear
        }
    }
}

/// Returns a bitboard consisting of the line fit between the two squares passed,
/// if there is such a line. If there isn't the empty bitboard will be returned.
///
/// The line returned notably goes beyond its start and end points.
///
/// # Examples
/// For the squares `A1` and `H8`, the line fit would be:
/// ```text
/// . . . . . . . X
/// . . . . . . 1 .
/// . . . . . 1 . .
/// . . . . 1 . . .
/// . . . 1 . . . .
/// . . 1 . . . . .
/// . 1 . . . . . .
/// X . . . . . . .
/// ```
///
/// For the squares `A2` and `H3` the line fit would be:
/// ```text
/// . . . . . . . .
/// . . . . . . . .
/// . . . . . . . .
/// . . . . . . . .
/// . . . . . . . .
/// . . . . . . . X
/// X . . . . . . .
/// . . . . . . . .
/// ```
///
/// Note that the `X`s represent the squares, and would generally have `1`s on them
/// (except in cases where there is no line fit).
pub(crate) fn line_fit(a: Square, b: Square) -> BitBoard {
    LINE[a.as_index() * 64 + b.as_index()]
}

/// Returns a bitboard consisting of the connecting line between the two squares passed,
/// if there is such a line. If there isn't the empty bitboard will be returned.
///
/// Unlike [`index::line_fit`], the line here doesn't go beyond the edge points.
///
/// # Examples
/// For the squares `B2` and `G7`, the line fit would be:
/// ```text
/// . . . . . . . .
/// . . . . . . X .
/// . . . . . 1 . .
/// . . . . 1 . . .
/// . . . 1 . . . .
/// . . 1 . . . . .
/// . X . . . . . .
/// . . . . . . . .
/// ```
///
/// For the squares `A2` and `H3` the line fit would be:
/// ```text
/// . . . . . . . .
/// . . . . . . . .
/// . . . . . . . .
/// . . . . . . . .
/// . . . . . . . .
/// . . . . . . . X
/// X . . . . . . .
/// . . . . . . . .
/// ```
///
/// Note that the `X`s represent the forming squares themselves, and these would never have `1`s
/// in them, as the line doesn't include its edge points.
pub(crate) fn line_between(a: Square, b: Square) -> BitBoard {
    BETWEEN[a.as_index() * 64 + b.as_index()]
}

/// Contains a variety of functions for generating Zobrist hashes for different parts of a board.
///
/// # Example
/// ```ignore
/// let white_king_rook = zobrist::piece(Piece::WHITE_ROOK, Square::WHITE_KING_ROOK);
/// let side_to_play = zobrist::side(Color::Black);
///
/// let combined_hash = white_king_rook ^ side_to_play;
/// ```
pub(crate) mod zobrist {
    use hash_bootstrap::{Color, Square};

    use crate::repr::{CastlingRights, ColoredPieceTable, Piece, PieceKind};

    use super::ZOBRIST_MAP;

    /// Generates the Zobrist hash for the given side. In a board this should be applied based on
    /// the currently playing player.
    pub(crate) fn side(color: Color) -> u64 {
        match color {
            Color::White => ZOBRIST_MAP.side.white_to_move,
            Color::Black => ZOBRIST_MAP.side.black_to_move,
        }
    }

    /// Generates the Zobrist hash for the file an en-passant is available on a particular file.
    /// This is used to distinguish boards beyond their piece configuration.
    ///
    /// If there is no such file, this shouldn't be applied.
    ///
    /// # Example
    /// Assuming a double-push happened on the E file we would have:
    /// ```ignore
    /// let e_file_en_passant_hash = zobrist::en_passant_file(4);
    /// ```
    ///
    /// # Panics
    /// This function panics if the passed file is invalid.
    pub(crate) fn en_passant_file(file: u8) -> u64 {
        ZOBRIST_MAP.ep_file[file as usize]
    }

    /// Generates the Zobrist hash for the castling rights of a board, to distinguish boards
    /// based on this.
    pub(crate) fn castling_rights(castling_rights: &CastlingRights) -> u64 {
        ZOBRIST_MAP.castling_rights.0[castling_rights.as_minimized_rights()]
    }

    /// Generates the Zobrist hash for a piece at a given square. Used in [`zobrist::piece_table`].
    ///
    /// # Example
    /// ```ignore
    /// let black_king = zobrist::piece(Piece::BLACK_KING, Square::BLACK_KING);
    /// ```
    pub(crate) fn piece(piece: Piece, square: Square) -> u64 {
        (match piece.kind {
            PieceKind::King => ZOBRIST_MAP.pieces.king,
            PieceKind::Queen => ZOBRIST_MAP.pieces.queen,
            PieceKind::Rook => ZOBRIST_MAP.pieces.rook,
            PieceKind::Bishop => ZOBRIST_MAP.pieces.bishop,
            PieceKind::Knight => ZOBRIST_MAP.pieces.knight,
            PieceKind::Pawn => ZOBRIST_MAP.pieces.pawn,
        })[square]
            .wrapping_mul(side(piece.color))
    }

    /// Generates the Zobrist hash for a [`ColoredPieceTable`], by using [`zobrist::piece`] on each
    /// piece in the table individually.
    pub(crate) fn piece_table(piece_table: &ColoredPieceTable) -> u64 {
        piece_table
            .pieces()
            .iter()
            .zip(Square::ALL)
            .filter_map(|(piece, square)| piece.map(|piece| self::piece(piece, square)))
            .reduce(|hash, current| hash ^ current)
            .unwrap()
    }
}

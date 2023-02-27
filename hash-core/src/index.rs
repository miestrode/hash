use hash_build::{
    BitBoard, Color, Square, ZobristCastlingRights, ZobristMap, ZobristPieces, ZobristSide,
};

use crate::repr::{CastlingRights, ColoredPieceTable, Piece, PieceKind};

#[derive(Clone, Copy, Debug)]
pub struct Metadata {
    pub offset: usize,
    pub mask: BitBoard,
    #[cfg(not(target_feature = "bmi2"))]
    pub magic: u64,
}

include!(concat!(env!("OUT_DIR"), "/table.rs"));

include!(concat!(env!("OUT_DIR"), "/out.rs"));

#[cfg(target_feature = "bmi2")]
include!(concat!(env!("OUT_DIR"), "/pext.rs"));

#[cfg(not(target_feature = "bmi2"))]
include!(concat!(env!("OUT_DIR"), "/magic.rs"));

#[cfg(target_feature = "bmi2")]
pub fn rook_slides(piece: Square, blockers: BitBoard) -> BitBoard {
    use std::arch::x86_64::_pext_u64;

    let metadata = CROSS_META[piece];

    *unsafe {
        CROSS_SLIDES
            .get_unchecked(metadata.offset + _pext_u64(blockers.0, metadata.mask.0) as usize)
    }
}

#[cfg(target_feature = "bmi2")]
pub fn bishop_slides(piece: Square, blockers: BitBoard) -> BitBoard {
    use std::arch::x86_64::_pext_u64;

    let metadata = DIAGONAL_META[piece];

    *unsafe {
        DIAGONAL_SLIDES
            .get_unchecked(metadata.offset + _pext_u64(blockers.0, metadata.mask.0) as usize)
    }
}

#[cfg(target_feature = "bmi2")]
pub fn separated_rook_slides(
    piece: Square,
    blockers: BitBoard,
) -> (BitBoard, BitBoard, BitBoard, BitBoard) {
    use std::arch::x86_64::_pext_u64;

    let metadata = CROSS_META[piece];

    *unsafe {
        SEPARATED_CROSS_SLIDES
            .get_unchecked(metadata.offset + _pext_u64(blockers.0, metadata.mask.0) as usize)
    }
}

#[cfg(target_feature = "bmi2")]
pub fn separated_bishop_slides(
    piece: Square,
    blockers: BitBoard,
) -> (BitBoard, BitBoard, BitBoard, BitBoard) {
    use std::arch::x86_64::_pext_u64;

    let metadata = DIAGONAL_META[piece];

    *unsafe {
        SEPARATED_DIAGONAL_SLIDES
            .get_unchecked(metadata.offset + _pext_u64(blockers.0, metadata.mask.0) as usize)
    }
}

#[cfg(not(target_feature = "bmi2"))]
pub fn rook_slides(piece: Square, blockers: BitBoard) -> BitBoard {
    let metadata = CROSS_META[piece];
    unsafe {
        SLIDES.get_unchecked(
            metadata.offset
                + ((blockers & metadata.mask).0.wrapping_mul(metadata.magic) >> (64 - 12)) as usize,
        )
    }
}

#[cfg(not(target_feature = "bmi2"))]
pub fn bishop_slides(piece: Square, blockers: BitBoard) -> BitBoard {
    let metadata = DIAGONAL_META[piece];
    unsafe {
        SLIDES.get_unchecked(
            metadata.offset
                + ((blockers & metadata.mask).0.wrapping_mul(metadata.magic) >> (64 - 9)) as usize,
        )
    }
}

#[cfg(not(target_feature = "bmi2"))]
pub fn separated_rook_slides(
    piece: Square,
    blockers: BitBoard,
) -> (BitBoard, BitBoard, BitBoard, BitBoard) {
    let metadata = CROSS_META[piece];
    unsafe {
        SEPARATED_SLIDES.get_unchecked(
            metadata.offset
                + ((blockers & metadata.mask).0.wrapping_mul(metadata.magic) >> (64 - 12)) as usize,
        )
    }
}

#[cfg(not(target_feature = "bmi2"))]
pub fn separated_bishop_slides(
    piece: Square,
    blockers: BitBoard,
) -> (BitBoard, BitBoard, BitBoard, BitBoard) {
    let metadata = DIAGONAL_META[piece];
    unsafe {
        SEPARATED_SLIDES.get_unchecked(
            metadata.offset
                + ((blockers & metadata.mask).0.wrapping_mul(metadata.magic) >> (64 - 9)) as usize,
        )
    }
}

pub fn knight_attacks(piece: Square) -> BitBoard {
    KNIGHT_ATTACKS[piece]
}

pub fn king_attacks(piece: Square) -> BitBoard {
    KING_ATTACKS[piece]
}

pub fn zobrist_side(color: Color) -> u64 {
    match color {
        Color::White => ZOBRIST_MAP.side.white_to_move,
        Color::Black => ZOBRIST_MAP.side.black_to_move,
    }
}

pub fn zobrist_ep_file(file: u32) -> u64 {
    ZOBRIST_MAP.ep_file[file as usize]
}

pub fn zobrist_castling_rights(castling_rights: &CastlingRights) -> u64 {
    ZOBRIST_MAP.castling_rights.0[castling_rights.as_minimized_rights()]
}

pub fn zobrist_piece(piece: Piece, square: Square) -> u64 {
    (match piece.kind {
        PieceKind::King => ZOBRIST_MAP.pieces.king,
        PieceKind::Queen => ZOBRIST_MAP.pieces.queen,
        PieceKind::Rook => ZOBRIST_MAP.pieces.rook,
        PieceKind::Bishop => ZOBRIST_MAP.pieces.bishop,
        PieceKind::Knight => ZOBRIST_MAP.pieces.knight,
        PieceKind::Pawn => ZOBRIST_MAP.pieces.pawn,
    })[square]
        * zobrist_side(piece.color)
}

// SAFETY: This function assumes the piece table comes from a valid board
pub unsafe fn zobrist_piece_table(piece_table: &ColoredPieceTable) -> u64 {
    // SAFETY: See above
    unsafe {
        piece_table
            .0
            .iter()
            .enumerate()
            .filter_map(|(square_index, piece)| {
                piece.map(|piece| zobrist_piece(piece, Square(square_index as u32)))
            })
            .reduce(|hash, current| hash ^ current)
            .unwrap_unchecked()
    }
}

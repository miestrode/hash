use before_build::{BitBoard, Metadata, Square};

include!(concat!(env!("OUT_DIR"), "/table.rs"));

#[cfg(target_feature = "bmi2")]
include!(concat!(env!("OUT_DIR"), "/pext.rs"));

#[cfg(not(target_feature = "bmi2"))]
include!(concat!(env!("OUT_DIR"), "/magic.rs"));

#[cfg(target_feature = "bmi2")]
pub fn rook_slides(piece: Square, blockers: BitBoard) -> BitBoard {
    use std::arch::x86_64::{_pdep_u64, _pext_u64};

    let metadata = CROSS_META[piece];

    BitBoard(unsafe {
        _pdep_u64(
            CROSS_SLIDES[metadata.offset + _pext_u64(blockers.0, metadata.mask.0) as usize] as u64,
            CROSS_RAYS[piece].0,
        )
    })
}

#[cfg(target_feature = "bmi2")]
pub fn bishop_slides(piece: Square, blockers: BitBoard) -> BitBoard {
    use std::arch::x86_64::{_pdep_u64, _pext_u64};

    let metadata = DIAGONAL_META[piece];

    BitBoard(unsafe {
        _pdep_u64(
            DIAGONAL_SLIDES[metadata.offset + _pext_u64(blockers.0, metadata.mask.0) as usize]
                as u64,
            DIAGONAL_RAYS[piece].0,
        )
    })
}

#[cfg(not(target_feature = "bmi2"))]
pub fn rook_slides(piece: Square, blockers: BitBoard) -> BitBoard {
    let metadata = CROSS_META[piece];
    SLIDES[metadata.offset
        + ((blockers & metadata.mask).0.wrapping_mul(metadata.magic) >> (64 - 12)) as usize]
}

#[cfg(not(target_feature = "bmi2"))]
pub fn bishop_slides(piece: Square, blockers: BitBoard) -> BitBoard {
    let metadata = DIAGONAL_META[piece];
    SLIDES[metadata.offset
        + ((blockers & metadata.mask).0.wrapping_mul(metadata.magic) >> (64 - 9)) as usize]
}

pub fn knight_attacks(piece: Square) -> BitBoard {
    KNIGHT_ATTACKS[piece]
}

pub fn king_attacks(piece: Square) -> BitBoard {
    KING_ATTACKS[piece]
}

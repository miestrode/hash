use hash_build::{BitBoard, Square};

#[derive(Clone, Copy, Debug)]
pub struct Metadata {
    pub offset: usize,
    pub mask: BitBoard,
    #[cfg(not(target_feature = "bmi2"))]
    pub magic: u64,
}

include!(concat!(env!("OUT_DIR"), "/table.rs"));

#[cfg(target_feature = "bmi2")]
include!(concat!(env!("OUT_DIR"), "/pext.rs"));

#[cfg(not(target_feature = "bmi2"))]
include!(concat!(env!("OUT_DIR"), "/magic.rs"));

#[cfg(target_feature = "bmi2")]
pub fn rook_slides(piece: Square, blockers: BitBoard) -> BitBoard {
    use std::arch::x86_64::_pext_u64;

    let metadata = CROSS_META[piece];

    unsafe { CROSS_SLIDES[metadata.offset + _pext_u64(blockers.0, metadata.mask.0) as usize] }
}

#[cfg(target_feature = "bmi2")]
pub fn bishop_slides(piece: Square, blockers: BitBoard) -> BitBoard {
    use std::arch::x86_64::_pext_u64;

    let metadata = DIAGONAL_META[piece];

    unsafe { DIAGONAL_SLIDES[metadata.offset + _pext_u64(blockers.0, metadata.mask.0) as usize] }
}

#[cfg(target_feature = "bmi2")]
pub fn separated_rook_slides(
    piece: Square,
    blockers: BitBoard,
) -> (BitBoard, BitBoard, BitBoard, BitBoard) {
    use std::arch::x86_64::_pext_u64;

    let metadata = CROSS_META[piece];

    SEPARATED_CROSS_SLIDES
        [metadata.offset + unsafe { _pext_u64(blockers.0, metadata.mask.0) } as usize]
}

#[cfg(target_feature = "bmi2")]
pub fn separated_bishop_slides(
    piece: Square,
    blockers: BitBoard,
) -> (BitBoard, BitBoard, BitBoard, BitBoard) {
    use std::arch::x86_64::_pext_u64;

    let metadata = DIAGONAL_META[piece];

    SEPARATED_DIAGONAL_SLIDES
        [metadata.offset + unsafe { _pext_u64(blockers.0, metadata.mask.0) } as usize]
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

#[cfg(not(target_feature = "bmi2"))]
pub fn separated_rook_slides(
    piece: Square,
    blockers: BitBoard,
) -> (BitBoard, BitBoard, BitBoard, BitBoard) {
    let metadata = CROSS_META[piece];
    SEPARATED_SLIDES[metadata.offset
        + ((blockers & metadata.mask).0.wrapping_mul(metadata.magic) >> (64 - 12)) as usize]
}

#[cfg(not(target_feature = "bmi2"))]
pub fn separated_bishop_slides(
    piece: Square,
    blockers: BitBoard,
) -> (BitBoard, BitBoard, BitBoard, BitBoard) {
    let metadata = DIAGONAL_META[piece];
    SEPARATED_SLIDES[metadata.offset
        + ((blockers & metadata.mask).0.wrapping_mul(metadata.magic) >> (64 - 9)) as usize]
}

pub fn knight_attacks(piece: Square) -> BitBoard {
    KNIGHT_ATTACKS[piece]
}

pub fn king_attacks(piece: Square) -> BitBoard {
    KING_ATTACKS[piece]
}

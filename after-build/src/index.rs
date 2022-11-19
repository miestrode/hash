use crate::bitboard::BitBoard;

include!(concat!(env!("OUT_DIR"),"/table.rs"));

pub fn gen_rook_moves(piece: Square, blockers: BitBoard) -> BitBoard {
    CROSS_OFFSETS[piece]
}

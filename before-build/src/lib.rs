#![feature(const_trait_impl, const_ops)]
mod bitboard;
mod square;

pub use {bitboard::*, square::*};

#[derive(Clone, Copy, Debug)]
pub struct Metadata {
    pub offset: usize,
    pub mask: BitBoard,
    #[cfg(not(target_feature = "bmi2"))]
    pub magic: u64,
}

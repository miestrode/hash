use std::{
    iter,
    ops::{Add, AddAssign, Not, Shl, Shr, Sub},
};

use crate::square::Square;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitBoard(pub u64);

// Called "partial", because it doesn't include the empty set as a subset of the relevant bitboard.
pub struct PartialSubsetIter {
    bitboard: BitBoard,
    subset: u64,
}

impl Iterator for PartialSubsetIter {
    type Item = BitBoard;

    fn next(&mut self) -> Option<Self::Item> {
        self.subset = self.subset.wrapping_sub(self.bitboard.0) & self.bitboard.0;

        if self.subset == 0 {
            None
        } else {
            Some(BitBoard(self.subset))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, 1_usize.checked_shl(self.bitboard.ones()))
    }
}

pub struct BitIter {
    bitboard: BitBoard,
}

impl Iterator for BitIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bitboard.is_empty() {
            None
        } else {
            Some(self.bitboard.pop_first_one())
        }
    }
}

pub enum Orientation {
    BottomToTop,
    TopToBottom,
}

impl Not for Orientation {
    type Output = Orientation;

    fn not(self) -> Self::Output {
        match self {
            Orientation::BottomToTop => Orientation::TopToBottom,
            Orientation::TopToBottom => Orientation::BottomToTop,
        }
    }
}

impl BitBoard {
    pub const EMPTY: Self = Self(0);
    pub const A_FILE: Self = Self(0x0101010101010101);
    pub const H_FILE: Self = Self(0x8080808080808080);
    pub const RANK_8: Self = Self(0xFF00000000000000);
    pub const RANK_1: Self = Self(0x00000000000000FF);

    pub fn is_empty(&self) -> bool {
        *self == Self::EMPTY
    }

    // NOTE: This function does include the empty set, unlike the partial subset iterator, and goes
    // from the empty set, to the improper subset (the bitboard given as input)
    pub fn subsets(&self) -> impl Iterator<Item = BitBoard> {
        iter::once(BitBoard::EMPTY).chain(PartialSubsetIter {
            bitboard: *self,
            subset: 0,
        })
    }

    pub fn bits(&self) -> impl Iterator<Item = Square> {
        BitIter { bitboard: *self }
    }

    pub fn pop_first_one(&mut self) -> Square {
        debug_assert!(!self.is_empty());

        let square = self.0.trailing_zeros();

        // PERF: Consider switching this to (self.0 & self.0 - 1), in case performance is better
        self.0 = self.0 ^ (1 << square);

        Square(square)
    }

    pub fn ones(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn move_one_up(self, o: Orientation) -> Self {
        match o {
            Orientation::BottomToTop => (self - Self::RANK_8) >> 8,
            Orientation::TopToBottom => (self - Self::RANK_1) << 8,
        }
    }

    pub fn move_one_down(self, o: Orientation) -> Self {
        self.move_one_up(!o)
    }

    pub fn move_one_right(self) -> Self {
        (self - Self::H_FILE) >> 1
    }

    pub fn move_one_left(self) -> Self {
        (self - Self::A_FILE) << 1
    }

    pub fn move_one_up_right(self, o: Orientation) -> Self {
        self.move_one_up(o).move_one_right()
    }

    pub fn move_one_up_left(self, o: Orientation) -> Self {
        self.move_one_up(o).move_one_left()
    }

    pub fn move_one_down_left(self, o: Orientation) -> Self {
        self.move_one_down(o).move_one_left()
    }

    pub fn move_one_down_right(self, o: Orientation) -> Self {
        self.move_one_down(o).move_one_right()
    }
}

impl Sub for BitBoard {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 & !rhs.0)
    }
}

impl const Add for BitBoard {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 | rhs.0)
    }
}

impl AddAssign for BitBoard {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

// NOTE:
// Wait, what? Yes, the shifts are flipped. This is because the bit board uses a little-endian
// rank-file representation, which basically means that the board bits are placed from left to
// right. Therefore, a left shift on a bit board would really be a right shift on a collection of
// bits.
impl Shl<u32> for BitBoard {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self::Output {
        BitBoard(self.0 >> rhs)
    }
}

impl Shr<u32> for BitBoard {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output {
        BitBoard(self.0 << rhs)
    }
}

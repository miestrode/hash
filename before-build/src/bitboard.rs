use std::{
    fmt::{self, Display, Formatter, Write},
    iter,
    ops::{Add, AddAssign, BitAnd, Not, Shl, Shr, Sub},
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
        self.0 ^= 1 << square;

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

    pub fn count_ones(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn get_bit(&self, square: Square) -> bool {
        *self & square.as_bitboard() != BitBoard::EMPTY
    }

    // Taken from https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating#Horizontal
    pub fn h_flip(self) -> Self {
        let k_1 = 0x5555555555555555;
        let k_2 = 0x3333333333333333;
        let k_4 = 0x0f0f0f0f0f0f0f0f;
        
        let mut x = self.0;
        x = ((x >> 1) & k_1) + 2 * (x & k_1);
        x = ((x >> 2) & k_2) + 4 * (x & k_2);
        
        BitBoard(((x >> 4) & k_4) + 16 * (x & k_4))
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Sub for BitBoard {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 & !rhs.0)
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl const Add for BitBoard {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 | rhs.0)
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 & rhs.0)
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
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
#[allow(clippy::suspicious_arithmetic_impl)]
impl Shl<u32> for BitBoard {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self::Output {
        BitBoard(self.0 >> rhs)
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Shr<u32> for BitBoard {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output {
        BitBoard(self.0 << rhs)
    }
}

impl Display for BitBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for row in 1..=8 {
            for column in 0..8 {
                if self.get_bit(Square((64 - row * 8) + column)) {
                    f.write_char('1')?;
                } else {
                    f.write_char('.')?;
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! bb {
    ($line0:tt $line1:tt $line2:tt $line3:tt $line4:tt $line5:tt $line6:tt $line7:tt) => {
        BitBoard(
            ($line0 << 56)
                | ($line1 << 48)
                | ($line2 << 40)
                | ($line3 << 32)
                | ($line4 << 24)
                | ($line5 << 16)
                | ($line6 << 8)
                | $line7,
        ).h_flip()
    };
}

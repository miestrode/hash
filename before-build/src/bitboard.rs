use std::{
    fmt::{self, Display, Formatter, Write},
    iter,
    ops::{Add, AddAssign, BitAnd, Not, Shl, Shr, Sub},
};

use crate::square::Square;

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
        )
        .h_flip()
    };
}

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

#[derive(Clone, Copy)]
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

    pub const FULL: Self = !Self::EMPTY;

    pub const A_FILE: Self = bb!(
        10000000
        10000000
        10000000
        10000000
        10000000
        10000000
        10000000
        10000000
    );

    pub const H_FILE: Self = bb!(
        00000001
        00000001
        00000001
        00000001
        00000001
        00000001
        00000001
        00000001
    );

    pub const EDGE_FILES: Self = Self::A_FILE + Self::H_FILE;

    pub const RANK_1: Self = bb!(
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        11111111
    );

    pub const RANK_2: Self = bb!(
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        11111111
        00000000
    );

    pub const BOTTOM_RANKS: Self = Self::RANK_1 + Self::RANK_2;

    pub const TOP_RANKS: Self = Self::RANK_7 + Self::RANK_8;

    pub const RANK_8: Self = bb!(
        11111111
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
    );

    pub const RANK_7: Self = bb!(
        00000000
        11111111
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
    );

    // Used to check both if a piece attacks a spot between the king and rook and if the space
    // between them is empty.
    pub const BOTTOM_KS_SPACE: Self = bb!(
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00001110
    );

    // Used to check if there are any pieces between the rook and king
    pub const BOTTOM_QS_MOVE_SPACE: Self = bb!(
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        01111000
    );

    // Used to check if there are any attacks between the king and the king's final spot
    pub const BOTTOM_QS_DANGER_SPACE: Self = bb!(
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00000000
        00111000
    );

    pub const TOP_KS_SPACE: Self = Self::BOTTOM_KS_SPACE.v_flip();

    pub const TOP_QS_MOVE_SPACE: Self = Self::BOTTOM_QS_MOVE_SPACE.v_flip();

    pub const TOP_QS_DANGER_SPACE: Self = Self::BOTTOM_QS_DANGER_SPACE.v_flip();

    pub const EDGE_RANKS: Self = Self::RANK_1 + Self::RANK_8;

    pub const PAWN_START_RANKS: Self = Self::RANK_2 + Self::RANK_7;

    pub fn ks_space(orientation: Orientation) -> Self {
        match orientation {
            Orientation::BottomToTop => Self::BOTTOM_KS_SPACE,
            Orientation::TopToBottom => Self::TOP_KS_SPACE,
        }
    }

    pub fn qs_move_space(orientation: Orientation) -> Self {
        match orientation {
            Orientation::BottomToTop => Self::BOTTOM_QS_MOVE_SPACE,
            Orientation::TopToBottom => Self::TOP_QS_MOVE_SPACE,
        }
    }

    pub fn qs_danger_space(orientation: Orientation) -> Self {
        match orientation {
            Orientation::BottomToTop => Self::BOTTOM_QS_DANGER_SPACE,
            Orientation::TopToBottom => Self::TOP_QS_DANGER_SPACE,
        }
    }

    pub fn is_full(&self) -> bool {
        *self == Self::FULL
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::EMPTY
    }

    pub fn isnt_empty(&self) -> bool {
        *self != Self::EMPTY
    }

    // NOTE: This function does include the empty set, unlike the partial subset iterator, and goes
    // from the empty set, to the improper subset (the bitboard given as input)
    pub fn subsets(&self) -> impl Iterator<Item = BitBoard> {
        iter::once(BitBoard::EMPTY).chain(PartialSubsetIter {
            bitboard: *self,
            subset: 0,
        })
    }

    pub fn bits(&self) -> BitIter {
        BitIter { bitboard: *self }
    }

    pub fn pop_first_one(&mut self) -> Square {
        debug_assert!(!self.is_empty());

        let square = self.0.trailing_zeros();

        // PERF: Consider switching this to (self.0 & self.0 - 1), in case performance is better
        self.0 ^= 1 << square;

        Square(square)
    }

    pub fn first_one_as_square(&self) -> Square {
        Square(self.0.trailing_zeros())
    }

    pub fn ones(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn move_one_up(self, orientation: Orientation) -> Self {
        match orientation {
            Orientation::BottomToTop => (self - Self::RANK_8) >> 8,
            Orientation::TopToBottom => (self - Self::RANK_1) << 8,
        }
    }

    pub fn move_two_up(self, orientation: Orientation) -> Self {
        match orientation {
            Orientation::BottomToTop => (self - Self::TOP_RANKS) >> 16,
            Orientation::TopToBottom => (self - Self::BOTTOM_RANKS) << 16,
        }
    }

    // TODO: Heavy doubt, but maybe just a match would be faster? This should be tested
    pub fn move_one_down(self, orientation: Orientation) -> Self {
        self.move_one_up(!orientation)
    }

    pub fn move_one_right(self, orientation: Orientation) -> Self {
        match orientation {
            Orientation::BottomToTop => (self - Self::H_FILE) >> 1,
            Orientation::TopToBottom => (self - Self::A_FILE) << 1,
        }
    }

    // TODO: Heavy doubt, but maybe just a match would be faster? This should be tested
    pub fn move_one_left(self, orientation: Orientation) -> Self {
        self.move_one_right(!orientation)
    }

    pub fn move_one_up_right(self, orientation: Orientation) -> Self {
        self.move_one_up(orientation).move_one_right(orientation)
    }

    pub fn move_one_up_left(self, orientation: Orientation) -> Self {
        self.move_one_up(orientation).move_one_left(orientation)
    }

    pub fn move_one_down_left(self, orientation: Orientation) -> Self {
        self.move_one_down(orientation).move_one_left(orientation)
    }

    pub fn move_one_down_right(self, orientation: Orientation) -> Self {
        self.move_one_down(orientation).move_one_right(orientation)
    }

    pub fn count_ones(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn get_bit(&self, square: Square) -> bool {
        *self & square.as_bitboard() != BitBoard::EMPTY
    }

    pub fn toggle_bit(&mut self, square: Square) {
        self.0 ^= square.as_bitboard().0;
    }

    // Taken from https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating#Horizontal
    pub const fn h_flip(self) -> Self {
        let k_1 = 0x5555555555555555;
        let k_2 = 0x3333333333333333;
        let k_4 = 0x0f0f0f0f0f0f0f0f;

        let mut x = self.0;
        x = ((x >> 1) & k_1) + 2 * (x & k_1);
        x = ((x >> 2) & k_2) + 4 * (x & k_2);

        Self(((x >> 4) & k_4) + 16 * (x & k_4))
    }

    pub const fn v_flip(self) -> Self {
        Self(self.0.swap_bytes())
    }

    /*
    1) When a one has a one below it, it becomes one.
    2) Otherwise, the cell stays as it is.

    In other words, we need the "or" function.
    */
    pub fn smear_ones_up(self, orientation: Orientation) -> Self {
        self.move_one_up(orientation) + self
    }
}

impl IntoIterator for BitBoard {
    type Item = Square;

    type IntoIter = BitIter;

    fn into_iter(self) -> Self::IntoIter {
        self.bits()
    }
}

impl const Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Sub for BitBoard {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self & !rhs
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl const Add for BitBoard {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
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

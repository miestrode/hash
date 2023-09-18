use std::{
    fmt::{Display, Write},
    hint,
    ops::{Index, IndexMut},
    str::FromStr,
};

use crate::{bitboard::BitBoard, Color};

#[derive(Eq, Clone, Copy, PartialEq, Debug)]
/// Represents a square on the Chess board. To construct a square, use one of the constants (such
/// as [`Square::A1`], [`Square::E4`], etc.). Alternatively some select functions return `Square`s,
/// such as [`BitBoard::pop_first_one`].
pub struct Square(pub(crate) u8);

impl Square {
    /// Numeric value for the A file. Corresponds to values returned from [`Square::file`].
    pub const A_FILE: u8 = 0;

    /// Numeric value for the B file. Corresponds to values returned from [`Square::file`].
    pub const B_FILE: u8 = 1;

    /// Numeric value for the C file. Corresponds to values returned from [`Square::file`].
    pub const C_FILE: u8 = 2;

    /// Numeric value for the D file. Corresponds to values returned from [`Square::file`].
    pub const D_FILE: u8 = 3;

    /// Numeric value for the E file. Corresponds to values returned from [`Square::file`].
    pub const E_FILE: u8 = 4;

    /// Numeric value for the F file. Corresponds to values returned from [`Square::file`].
    pub const F_FILE: u8 = 5;

    /// Numeric value for the G file. Corresponds to values returned from [`Square::file`].
    pub const G_FILE: u8 = 6;

    /// Numeric value for the H file. Corresponds to values returned from [`Square::file`].
    pub const H_FILE: u8 = 7;

    /// Numeric value for the first rank. Corresponds to values returned from [`Square::rank`].
    pub const RANK_1: u8 = 0;

    /// Numeric value for the second rank. Corresponds to values returned from [`Square::rank`].
    pub const RANK_2: u8 = 1;

    /// Numeric value for the third rank. Corresponds to values returned from [`Square::rank`].
    pub const RANK_3: u8 = 2;

    /// Numeric value for the fourth rank. Corresponds to values returned from [`Square::rank`].
    pub const RANK_4: u8 = 3;

    /// Numeric value for the fifth rank. Corresponds to values returned from [`Square::rank`].
    pub const RANK_5: u8 = 4;

    /// Numeric value for the sixth rank. Corresponds to values returned from [`Square::rank`].
    pub const RANK_6: u8 = 5;

    /// Numeric value for the seventh rank. Corresponds to values returned from [`Square::rank`].
    pub const RANK_7: u8 = 6;

    /// Numeric value for the eighth rank. Corresponds to values returned from [`Square::rank`].
    pub const RANK_8: u8 = 7;

    pub const A1: Square = Square(0b000000);
    pub const B1: Square = Square(0b000001);
    pub const C1: Square = Square(0b000010);
    pub const D1: Square = Square(0b000011);
    pub const E1: Square = Square(0b000100);
    pub const F1: Square = Square(0b000101);
    pub const G1: Square = Square(0b000110);
    pub const H1: Square = Square(0b000111);
    pub const A2: Square = Square(0b001000);
    pub const B2: Square = Square(0b001001);
    pub const C2: Square = Square(0b001010);
    pub const D2: Square = Square(0b001011);
    pub const E2: Square = Square(0b001100);
    pub const F2: Square = Square(0b001101);
    pub const G2: Square = Square(0b001110);
    pub const H2: Square = Square(0b001111);
    pub const A3: Square = Square(0b010000);
    pub const B3: Square = Square(0b010001);
    pub const C3: Square = Square(0b010010);
    pub const D3: Square = Square(0b010011);
    pub const E3: Square = Square(0b010100);
    pub const F3: Square = Square(0b010101);
    pub const G3: Square = Square(0b010110);
    pub const H3: Square = Square(0b010111);
    pub const A4: Square = Square(0b011000);
    pub const B4: Square = Square(0b011001);
    pub const C4: Square = Square(0b011010);
    pub const D4: Square = Square(0b011011);
    pub const E4: Square = Square(0b011100);
    pub const F4: Square = Square(0b011101);
    pub const G4: Square = Square(0b011110);
    pub const H4: Square = Square(0b011111);
    pub const A5: Square = Square(0b100000);
    pub const B5: Square = Square(0b100001);
    pub const C5: Square = Square(0b100010);
    pub const D5: Square = Square(0b100011);
    pub const E5: Square = Square(0b100100);
    pub const F5: Square = Square(0b100101);
    pub const G5: Square = Square(0b100110);
    pub const H5: Square = Square(0b100111);
    pub const A6: Square = Square(0b101000);
    pub const B6: Square = Square(0b101001);
    pub const C6: Square = Square(0b101010);
    pub const D6: Square = Square(0b101011);
    pub const E6: Square = Square(0b101100);
    pub const F6: Square = Square(0b101101);
    pub const G6: Square = Square(0b101110);
    pub const H6: Square = Square(0b101111);
    pub const A7: Square = Square(0b110000);
    pub const B7: Square = Square(0b110001);
    pub const C7: Square = Square(0b110010);
    pub const D7: Square = Square(0b110011);
    pub const E7: Square = Square(0b110100);
    pub const F7: Square = Square(0b110101);
    pub const G7: Square = Square(0b110110);
    pub const H7: Square = Square(0b110111);
    pub const A8: Square = Square(0b111000);
    pub const B8: Square = Square(0b111001);
    pub const C8: Square = Square(0b111010);
    pub const D8: Square = Square(0b111011);
    pub const E8: Square = Square(0b111100);
    pub const F8: Square = Square(0b111101);
    pub const G8: Square = Square(0b111110);
    pub const H8: Square = Square(0b111111);

    /// An array of all the squares of a Chess board, arranged in left-to-right, bottom-to-top
    /// order. This means that, for example `Square::ALL[0] == Square::A1`
    /// and in general `Square::ALL[test_square] == test_square`.
    pub const ALL: [Square; 64] = [
        Self::A1,
        Self::B1,
        Self::C1,
        Self::D1,
        Self::E1,
        Self::F1,
        Self::G1,
        Self::H1,
        Self::A2,
        Self::B2,
        Self::C2,
        Self::D2,
        Self::E2,
        Self::F2,
        Self::G2,
        Self::H2,
        Self::A3,
        Self::B3,
        Self::C3,
        Self::D3,
        Self::E3,
        Self::F3,
        Self::G3,
        Self::H3,
        Self::A4,
        Self::B4,
        Self::C4,
        Self::D4,
        Self::E4,
        Self::F4,
        Self::G4,
        Self::H4,
        Self::A5,
        Self::B5,
        Self::C5,
        Self::D5,
        Self::E5,
        Self::F5,
        Self::G5,
        Self::H5,
        Self::A6,
        Self::B6,
        Self::C6,
        Self::D6,
        Self::E6,
        Self::F6,
        Self::G6,
        Self::H6,
        Self::A7,
        Self::B7,
        Self::C7,
        Self::D7,
        Self::E7,
        Self::F7,
        Self::G7,
        Self::H7,
        Self::A8,
        Self::B8,
        Self::C8,
        Self::D8,
        Self::E8,
        Self::F8,
        Self::G8,
        Self::H8,
    ];

    /// The location of queen rook of white, at the starting position.
    pub const WHITE_QUEEN_ROOK: Self = Self::A1;

    /// The location of queen rook of black, at the starting position.
    pub const BLACK_QUEEN_ROOK: Self = Self::A8;

    /// The location of king rook of white, at the starting position.
    pub const WHITE_KING_ROOK: Self = Self::H1;

    /// The location of king rook of black, at the starting position.
    pub const BLACK_KING_ROOK: Self = Self::H8;

    /// The location of the white king at the starting position.
    pub const WHITE_KING: Self = Self::E1;

    /// The location of the black king at the starting position.
    pub const BLACK_KING: Self = Self::E8;

    /// Moves the square down by one rank, without any coherence checks.
    ///
    /// # Example
    /// ```
    /// # use hash_bootstrap::{Color, Square};
    ///
    /// let square = Square::F4;
    ///
    /// assert_eq!(unsafe { square.move_one_down_unchecked(Color::White) }, Square::F3);
    /// ```
    ///
    /// # Safety
    /// The move is assumed to be valid on the square - meaning that it makes logical sense.
    /// Moving a square on the first rank one rank down, is for example illogical.
    pub unsafe fn move_one_down_unchecked(&self, color: Color) -> Self {
        match color {
            Color::White => Self(self.0 - 8),
            Color::Black => Self(self.0 + 8),
        }
    }

    /// Moves the square down by two rank, without any coherence checks.
    ///
    /// # Example
    /// ```
    /// # use hash_bootstrap::{Color, Square};
    ///
    /// let square = Square::F4;
    ///
    /// assert_eq!(unsafe { square.move_two_down_unchecked(Color::White) }, Square::F2);
    /// ```
    ///
    /// # Safety
    /// The move is assumed to be valid on the square - meaning that it makes logical sense.
    /// Moving a square on the first rank one rank down, is for example illogical.
    pub unsafe fn move_two_down_unchecked(&self, color: Color) -> Self {
        match color {
            Color::White => Self(self.0 - 16),
            Color::Black => Self(self.0 + 16),
        }
    }

    /// Moves the square down by one rank and one file, without any coherence checks.
    ///
    /// # Example
    /// ```
    /// # use hash_bootstrap::{Color, Square};
    ///
    /// let square = Square::F4;
    ///
    /// assert_eq!(unsafe { square.move_one_down_left_unchecked(Color::White) }, Square::E3);
    /// ```
    ///
    /// # Safety
    /// The move is assumed to be valid on the square - meaning that it makes logical sense.
    /// Moving a square on the first rank one move down, is for example illogical.
    pub unsafe fn move_one_down_left_unchecked(&self, color: Color) -> Self {
        match color {
            Color::White => Self(self.0 - 9),
            Color::Black => Self(self.0 + 9),
        }
    }

    /// Moves the square down by one rank, and up by one file, without any coherence checks.
    ///
    /// # Example
    /// ```
    /// # use hash_bootstrap::{Color, Square};
    ///
    /// let square = Square::F4;
    ///
    /// assert_eq!(unsafe { square.move_one_down_right_unchecked(Color::White) }, Square::G3);
    /// ```
    ///
    /// # Safety
    /// The move is assumed to be valid on the square - meaning that it makes logical sense.
    /// Moving a square on the first rank one move down, is for example illogical.
    pub unsafe fn move_one_down_right_unchecked(&self, color: Color) -> Self {
        match color {
            Color::White => Self(self.0 - 7),
            Color::Black => Self(self.0 + 7),
        }
    }

    /// Moves the square up by one rank, without any coherence checks.
    ///
    /// # Example
    /// ```
    /// # use hash_bootstrap::{Color, Square};
    ///
    /// let square = Square::F4;
    ///
    /// assert_eq!(unsafe { square.move_one_up_unchecked(Color::White) }, Square::F5);
    /// ```
    ///
    /// # Safety
    /// The move is assumed to be valid on the square - meaning that it makes logical sense.
    /// Moving a square on the first rank one move down, is for example illogical.
    pub unsafe fn move_one_up_unchecked(&self, color: Color) -> Self {
        match color {
            Color::White => Self(self.0 + 8),
            Color::Black => Self(self.0 - 8),
        }
    }

    /// Gets the rank of the square, as a number from `0` to `7`.
    /// The first rank gets number `0`, the second `1`, and so on.
    ///
    /// # Example
    /// ```
    /// # use hash_bootstrap::Square;
    ///
    /// let king = Square::WHITE_KING;
    ///
    /// assert_eq!(king.rank(), Square::RANK_1);
    /// ```
    pub fn rank(&self) -> u8 {
        self.0 / 8
    }

    /// Gets the file of the square, as a number from `0` to `7`.
    /// The A file gets number `0`, the B file `1`, and so on.
    ///
    /// # Example
    /// ```
    /// # use hash_bootstrap::Square;
    ///
    /// let king = Square::BLACK_KING;
    ///
    /// assert_eq!(king.file(), Square::E_FILE);
    /// ```
    pub fn file(&self) -> u8 {
        self.0 % 8
    }

    /// Gets the square, as a `usize` value. When you use this value to index into `Square::ALL`,
    /// you will get the square you used to get this value.
    ///
    /// # Example
    /// ```
    /// # use hash_bootstrap::Square;
    ///
    /// let square = Square::A1;
    ///
    /// assert_eq!(square.as_index(), 0);
    /// ```
    pub fn as_index(&self) -> usize {
        self.0 as usize
    }

    /// Checks if the current square forms a line with the provided `other` square. A line can be
    /// horizontal, vertical or diagonal.
    ///
    /// # Example
    /// ```rust
    /// # use hash_bootstrap::Square;
    ///
    /// let start = Square::A1;
    /// let end = Square::H8;
    ///
    /// assert!(start.on_line_with(end));
    /// ```
    pub fn on_line_with(&self, other: Self) -> bool {
        self.rank() == other.rank()
            || self.file() == other.file()
            || self.file().abs_diff(other.file()) == self.rank().abs_diff(other.rank())
    }

    /// Checks if the current square exists in the rectangle formed by the two passed squares `a`
    /// and `b`, that doesn't include `a` and `b`.
    ///
    /// This means that for example for squares C3 and F6 the resulting rectangle is:
    /// ```text
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . X . .
    /// . . . 1 1 . . .
    /// . . . 1 1 . . .
    /// . . X . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// ```
    ///
    /// Where the `X`s are the two squares forming the rectangle.
    ///
    /// # Example
    /// ```rust
    /// # use hash_bootstrap::Square;
    ///
    /// let start = Square::C3;
    /// let end = Square::F6;
    ///
    /// assert!(Square::E5.in_rectangle(start, end));
    /// ```
    pub fn in_rectangle(&self, a: Square, b: Square) -> bool {
        (a.file() + 1..b.file()).contains(&self.file())
            && (a.rank() + 1..b.rank()).contains(&self.rank())
    }
}

impl From<Square> for BitBoard {
    fn from(value: Square) -> Self {
        BitBoard(1 << value.0)
    }
}

impl TryFrom<u8> for Square {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (0..64).contains(&value) {
            Ok(Square(value))
        } else {
            Err("Cannot convert an index not between 0 to 63 to a square")
        }
    }
}

impl TryFrom<BitBoard> for Square {
    type Error = &'static str;

    fn try_from(value: BitBoard) -> Result<Self, Self::Error> {
        if value.is_a_single_one() {
            Ok(value.first_one_as_square().unwrap())
        } else {
            Err("Cannot convert non-single-one bitboard to a square")
        }
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self.file() {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            // SAFETY: The file must be from 0 to 7
            _ => unsafe { hint::unreachable_unchecked() },
        })?;
        f.write_fmt(format_args!("{}", self.rank() + 1))
    }
}

impl<T> Index<Square> for [T; 64] {
    type Output = T;

    fn index(&self, square: Square) -> &Self::Output {
        unsafe { self.get_unchecked(square.as_index()) }
    }
}

impl<T> IndexMut<Square> for [T; 64] {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index.as_index()) }
    }
}

impl FromStr for Square {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            Err("Input must contain two characters")
        } else {
            let mut characters = s.chars();

            Ok(Self(
                match characters.next().unwrap() {
                    'a' => 0,
                    'b' => 1,
                    'c' => 2,
                    'd' => 3,
                    'e' => 4,
                    'f' => 5,
                    'g' => 6,
                    'h' => 7,
                    _ => return Err("Input's column descriptor must be a character from a to h"),
                } + match characters.next().unwrap() {
                    '1' => 0,
                    '2' => 8,
                    '3' => 16,
                    '4' => 24,
                    '5' => 32,
                    '6' => 40,
                    '7' => 48,
                    '8' => 56,
                    _ => return Err("Input's row descriptor must be a digit from 1 to 8"),
                },
            ))
        }
    }
}

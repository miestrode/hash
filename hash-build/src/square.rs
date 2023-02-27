use std::{
    fmt::{Display, Write},
    hint::unreachable_unchecked,
    ops::{Index, IndexMut},
    str::FromStr,
};

use crate::{bitboard::BitBoard, Color};

#[derive(Eq, Hash, Clone, Copy, PartialEq)]
pub struct Square(pub u32);

impl Square {
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

    pub const BOTTOM_LEFT_ROOK: Self = Self::A1;
    pub const TOP_LEFT_ROOK: Self = Self::A8;

    pub const BOTTOM_RIGHT_ROOK: Self = Self::H1;
    pub const TOP_RIGHT_ROOK: Self = Self::H8;

    pub const BOTTOM_KING: Self = Self::E1;
    pub const TOP_KING: Self = Self::E8;

    pub fn as_bitboard(&self) -> BitBoard {
        BitBoard(1 << self.0)
    }

    pub unsafe fn move_one_down_unchecked(&self, color: Color) -> Self {
        match color {
            Color::White => Self(self.0 - 8),
            Color::Black => Self(self.0 + 8),
        }
    }

    pub unsafe fn move_two_down_unchecked(&self, color: Color) -> Self {
        match color {
            Color::White => Self(self.0 - 16),
            Color::Black => Self(self.0 + 16),
        }
    }

    pub unsafe fn move_one_down_left_unchecked(&self, color: Color) -> Self {
        match color {
            Color::White => Self(self.0 - 9),
            Color::Black => Self(self.0 + 9),
        }
    }

    pub unsafe fn move_one_down_right_unchecked(&self, color: Color) -> Self {
        match color {
            Color::White => Self(self.0 - 7),
            Color::Black => Self(self.0 + 7),
        }
    }

    pub unsafe fn move_one_up_unchecked(&self, color: Color) -> Self {
        match color {
            Color::White => Self(self.0 + 8),
            Color::Black => Self(self.0 - 8),
        }
    }

    pub fn rank(&self) -> u32 {
        self.0 / 8
    }

    pub fn file(&self) -> u32 {
        self.0 % 8
    }

    pub fn as_index(&self) -> usize {
        self.0 as usize
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
            _ => unsafe { unreachable_unchecked() },
        })?;
        f.write_fmt(format_args!("{}", self.rank() + 1))
    }
}

impl<T> Index<Square> for [T; 64] {
    type Output = T;

    fn index(&self, index: Square) -> &Self::Output {
        unsafe { self.get_unchecked(index.0 as usize) }
    }
}

impl<T> IndexMut<Square> for [T; 64] {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index.0 as usize) }
    }
}
impl<T> Index<Square> for Vec<T> {
    type Output = T;

    fn index(&self, index: Square) -> &Self::Output {
        unsafe { self.get_unchecked(index.0 as usize) }
    }
}

impl<T> IndexMut<Square> for Vec<T> {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index.0 as usize) }
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

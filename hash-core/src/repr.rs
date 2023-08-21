use std::{
    fmt::{self, Display, Write},
    str::FromStr,
};

use crate::{BitBoard, Color, Square};

#[derive(Eq, Hash, Debug, Clone, Copy, PartialEq)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl Display for PieceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            PieceKind::King => 'k',
            PieceKind::Queen => 'q',
            PieceKind::Rook => 'r',
            PieceKind::Bishop => 'b',
            PieceKind::Knight => 'n',
            PieceKind::Pawn => 'p',
        })
    }
}

impl PieceKind {
    pub const PROMOTIONS: [Self; 4] = [Self::Queen, Self::Rook, Self::Bishop, Self::Knight];
}

#[derive(Clone, Copy)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
}

impl Piece {
    pub const WHITE_PAWN: Self = Self {
        kind: PieceKind::Pawn,
        color: Color::White,
    };
    pub const WHITE_KNIGHT: Self = Self {
        kind: PieceKind::Knight,
        color: Color::White,
    };
    pub const WHITE_BISHOP: Self = Self {
        kind: PieceKind::Bishop,
        color: Color::White,
    };
    pub const WHITE_ROOK: Self = Self {
        kind: PieceKind::Rook,
        color: Color::White,
    };
    pub const WHITE_QUEEN: Self = Self {
        kind: PieceKind::Queen,
        color: Color::White,
    };
    pub const WHITE_KING: Self = Self {
        kind: PieceKind::King,
        color: Color::White,
    };

    pub const BLACK_PAWN: Self = Self {
        kind: PieceKind::Pawn,
        color: Color::Black,
    };
    pub const BLACK_KNIGHT: Self = Self {
        kind: PieceKind::Knight,
        color: Color::Black,
    };
    pub const BLACK_BISHOP: Self = Self {
        kind: PieceKind::Bishop,
        color: Color::Black,
    };
    pub const BLACK_ROOK: Self = Self {
        kind: PieceKind::Rook,
        color: Color::Black,
    };
    pub const BLACK_QUEEN: Self = Self {
        kind: PieceKind::Queen,
        color: Color::Black,
    };
    pub const BLACK_KING: Self = Self {
        kind: PieceKind::King,
        color: Color::Black,
    };
}

impl Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.color {
            Color::White => match self.kind {
                PieceKind::King => 'K',
                PieceKind::Queen => 'Q',
                PieceKind::Rook => 'R',
                PieceKind::Bishop => 'B',
                PieceKind::Knight => 'N',
                PieceKind::Pawn => 'P',
            },
            Color::Black => match self.kind {
                PieceKind::King => 'k',
                PieceKind::Queen => 'q',
                PieceKind::Rook => 'r',
                PieceKind::Bishop => 'b',
                PieceKind::Knight => 'n',
                PieceKind::Pawn => 'p',
            },
        }
        .fmt(f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveMeta {
    Promotion(PieceKind),
    EnPassant,
    DoublePush,
    CastleKs,
    CastleQs,
    None,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct Move {
    pub origin: Square,
    pub target: Square,
    pub moved_piece_kind: PieceKind,
    pub meta: MoveMeta,
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}{}", self.origin, self.target))?;

        if let MoveMeta::Promotion(kind) = self.meta {
            kind.fmt(f)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pins {
    pub horizontal: BitBoard,
    pub vertical: BitBoard,
    pub diagonal: BitBoard,
    pub anti_diagonal: BitBoard,
}

// TODO: Make sure all the of movement functions are cached
impl Pins {
    pub const EMPTY: Self = Self {
        horizontal: BitBoard::EMPTY,
        vertical: BitBoard::EMPTY,
        diagonal: BitBoard::EMPTY,
        anti_diagonal: BitBoard::EMPTY,
    };

    // Returns a bitboard for all pieces capable of psuedo-moving vertically
    pub fn vertical_movement(&self) -> BitBoard {
        !(self.horizontal + self.diagonal + self.anti_diagonal)
    }

    pub fn diagonal_movement(&self) -> BitBoard {
        !(self.horizontal + self.vertical + self.anti_diagonal)
    }

    pub fn anti_diagonal_movement(&self) -> BitBoard {
        !(self.vertical + self.diagonal + self.horizontal)
    }

    pub fn all(&self) -> BitBoard {
        self.vertical + self.horizontal + self.diagonal + self.anti_diagonal
    }

    pub fn cross_pins(&self) -> BitBoard {
        self.vertical + self.horizontal
    }

    pub fn diagonal_pins(&self) -> BitBoard {
        self.diagonal + self.anti_diagonal
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastlingRights(pub [bool; 64]);

impl CastlingRights {
    pub fn empty() -> Self {
        let table = [false; 64];

        Self(table)
    }

    pub fn can_castle_ks(&self) -> bool {
        (self.0[Square::E1] ^ self.0[Square::E8]) && (self.0[Square::H1] ^ self.0[Square::H8])
    }

    pub fn can_castle_qs(&self) -> bool {
        (self.0[Square::E1] ^ self.0[Square::E8]) && (self.0[Square::A1] ^ self.0[Square::A8])
    }

    pub fn as_minimized_rights(&self) -> usize {
        self.0[Square::A1] as usize
            | ((self.0[Square::H1] as usize) << 1)
            | ((self.0[Square::A8] as usize) << 2)
            | ((self.0[Square::H8] as usize) << 3)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Player {
    pub king: BitBoard,
    pub queens: BitBoard,
    pub rooks: BitBoard,
    pub bishops: BitBoard,
    pub knights: BitBoard,
    pub pawns: BitBoard,
    pub dangers: BitBoard,       // Positions the enemy king could be eaten at
    pub valid_targets: BitBoard, // Valid target positions for moves. Used when at check
    pub pins: Pins,
    pub occupation: BitBoard, // All of the squares occupied by this player
    pub king_must_move: bool, // A flag representing whether this turn, the king needs to move
    pub castling_rights: CastlingRights,
}

impl Player {
    pub fn blank() -> Self {
        Self {
            king: BitBoard::EMPTY,
            queens: BitBoard::EMPTY,
            rooks: BitBoard::EMPTY,
            bishops: BitBoard::EMPTY,
            knights: BitBoard::EMPTY,
            pawns: BitBoard::EMPTY,
            dangers: BitBoard::EMPTY,
            valid_targets: BitBoard::FULL,
            pins: Pins::EMPTY,
            occupation: BitBoard::EMPTY,
            king_must_move: false,
            castling_rights: CastlingRights::empty(),
        }
    }

    pub fn piece_bitboard_mut(&mut self, kind: PieceKind) -> &mut BitBoard {
        match kind {
            PieceKind::King => &mut self.king,
            PieceKind::Queen => &mut self.queens,
            PieceKind::Rook => &mut self.rooks,
            PieceKind::Bishop => &mut self.bishops,
            PieceKind::Knight => &mut self.knights,
            PieceKind::Pawn => &mut self.pawns,
        }
    }

    pub unsafe fn move_piece_unchecked(&mut self, kind: PieceKind, origin: Square, target: Square) {
        let pieces = self.piece_bitboard_mut(kind);
        pieces.toggle_bit(origin);
        pieces.toggle_bit(target);

        self.occupation.toggle_bit(origin);
        self.occupation.toggle_bit(target);
    }

    pub fn toggle_piece(&mut self, kind: PieceKind, square: Square) {
        self.piece_bitboard_mut(kind).toggle_bit(square);
        self.occupation.toggle_bit(square);
    }

    pub fn is_in_check(&self) -> bool {
        !self.valid_targets.is_full()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EpData {
    pub capture_point: BitBoard,
    pub pawn: Square,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PieceTable(pub [Option<PieceKind>; 64]);

impl PieceTable {
    pub fn move_piece(&mut self, origin: Square, target: Square) {
        self.0.swap(origin.as_index(), target.as_index());
        self.set(None, origin);
    }

    pub fn piece_kind(&self, square: Square) -> Option<PieceKind> {
        self.0[square]
    }

    pub fn set(&mut self, kind: Option<PieceKind>, square: Square) {
        self.0[square] = kind;
    }
}

pub struct ColoredPieceTable(pub [Option<Piece>; 64]);

impl ColoredPieceTable {
    pub const EMPTY: Self = Self([None; 64]);

    pub fn uncolored(&self) -> PieceTable {
        PieceTable(
            self.0
                .into_iter()
                .map(|square| square.map(|piece| piece.kind))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }
}

impl FromStr for ColoredPieceTable {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut board_pieces = ColoredPieceTable::EMPTY;

        let rows = s.split('/').collect::<Vec<_>>();

        if rows.len() != 8 {
            Err("Input contains the wrong amount of rows")
        } else {
            let mut row_offset = 64;

            for row in rows {
                row_offset -= 8;
                let mut column_offset = -1; // This goes from 0-7, so we want to make sure the first increase puts us at index 0.

                for character in row.chars() {
                    match character {
                        '1' => column_offset += 1,
                        '2' => column_offset += 2,
                        '3' => column_offset += 3,
                        '4' => column_offset += 4,
                        '5' => column_offset += 5,
                        '6' => column_offset += 6,
                        '7' => column_offset += 7,
                        '8' => column_offset += 8,
                        _ => {
                            column_offset += 1;
                            board_pieces.0[(row_offset + column_offset) as usize] =
                                Some(match character {
                                    'K' => Piece::WHITE_KING,
                                    'Q' => Piece::WHITE_QUEEN,
                                    'R' => Piece::WHITE_ROOK,
                                    'B' => Piece::WHITE_BISHOP,
                                    'N' => Piece::WHITE_KNIGHT,
                                    'P' => Piece::WHITE_PAWN,
                                    'k' => Piece::BLACK_KING,
                                    'q' => Piece::BLACK_QUEEN,
                                    'r' => Piece::BLACK_ROOK,
                                    'b' => Piece::BLACK_BISHOP,
                                    'n' => Piece::BLACK_KNIGHT,
                                    'p' => Piece::BLACK_PAWN,
                                    _ => return Err(
                                        "Input contains an invalid character in one of the rows",
                                    ),
                                });
                        }
                    }

                    if column_offset > 7 {
                        return Err(
                            "Input contains an overflowed row (The column offset is too high)",
                        );
                    }
                }
            }

            Ok(board_pieces)
        }
    }
}

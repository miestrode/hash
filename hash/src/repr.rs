use std::{
    fmt::{self, Display, Write},
    str::FromStr,
};
use hash_bootstrap::{BitBoard, Color, Square};

#[derive(Eq, Hash, Debug, Clone, Copy, PartialEq)]
/// Represents a type of piece, such as a [king](`PieceKind::King`),
/// or a [queen](`PieceKind::Queen`).
pub(crate) enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl Display for PieceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl FromStr for PieceKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 1 {
            Ok(match &s[0..1] {
                "k" => PieceKind::King,
                "q" => PieceKind::Queen,
                "r" => PieceKind::Rook,
                "b" => PieceKind::Bishop,
                "n" => PieceKind::Knight,
                "p" => PieceKind::Pawn,
                _ => return Err("Input must be a valid piece type character (k, q, r, b, n, p)"),
            })
        } else {
            Err("Input must be a single character")
        }
    }
}

impl PieceKind {
    /// An array of each piece a pawn can promote to.
    pub(crate) const PROMOTIONS: [Self; 4] = [Self::Queen, Self::Rook, Self::Bishop, Self::Knight];
}

#[derive(Clone, Copy, PartialEq)]
/// Represents a Chess piece, which has a [type](`PieceKind`) and a [color](`Color`).
pub(crate) struct Piece {
    pub(crate) kind: PieceKind,
    pub(crate) color: Color,
}

impl Piece {
    pub(crate) const WHITE_PAWN: Self = Self {
        kind: PieceKind::Pawn,
        color: Color::White,
    };

    pub(crate) const WHITE_KNIGHT: Self = Self {
        kind: PieceKind::Knight,
        color: Color::White,
    };

    pub(crate) const WHITE_BISHOP: Self = Self {
        kind: PieceKind::Bishop,
        color: Color::White,
    };

    pub(crate) const WHITE_ROOK: Self = Self {
        kind: PieceKind::Rook,
        color: Color::White,
    };

    pub(crate) const WHITE_QUEEN: Self = Self {
        kind: PieceKind::Queen,
        color: Color::White,
    };

    pub(crate) const WHITE_KING: Self = Self {
        kind: PieceKind::King,
        color: Color::White,
    };

    pub(crate) const BLACK_PAWN: Self = Self {
        kind: PieceKind::Pawn,
        color: Color::Black,
    };

    pub(crate) const BLACK_KNIGHT: Self = Self {
        kind: PieceKind::Knight,
        color: Color::Black,
    };

    pub(crate) const BLACK_BISHOP: Self = Self {
        kind: PieceKind::Bishop,
        color: Color::Black,
    };

    pub(crate) const BLACK_ROOK: Self = Self {
        kind: PieceKind::Rook,
        color: Color::Black,
    };

    pub(crate) const BLACK_QUEEN: Self = Self {
        kind: PieceKind::Queen,
        color: Color::Black,
    };

    pub(crate) const BLACK_KING: Self = Self {
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

#[derive(PartialEq, Eq, Clone, Copy)]
/// Represents a move in the game of Chess. To create a move one can use [`Board::interpret_move`].
pub struct Move {
    pub(crate) origin: Square,
    pub(crate) target: Square,
    pub(crate) promotion: Option<PieceKind>,
}

impl Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.origin.fmt(f)?;
        self.target.fmt(f)?;

        if let Some(kind) = self.promotion {
            kind.fmt(f)?;
        }

        Ok(())
    }
}

impl FromStr for Move {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 4 {
            Err("Input too short")
        } else if s.len() > 5 {
            Err("Input too long")
        } else {
            let origin = Square::from_str(&s[0..2])?;
            let target = Square::from_str(&s[2..4])?;

            let promotion = if s.len() == 5 {
                Some(PieceKind::from_str(&s[4..5])?)
            } else {
                None
            };

            Ok(Move {
                origin,
                target,
                promotion,
            })
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct CastlingRights(pub [bool; 64]);

impl CastlingRights {
    pub(crate) fn empty() -> Self {
        Self([false; 64])
    }

    pub(crate) fn can_castle_king_side(&self) -> bool {
        (self.0[Square::E1] ^ self.0[Square::E8]) && (self.0[Square::H1] ^ self.0[Square::H8])
    }

    pub(crate) fn can_castle_queen_side(&self) -> bool {
        (self.0[Square::E1] ^ self.0[Square::E8]) && (self.0[Square::A1] ^ self.0[Square::A8])
    }

    pub(crate) fn as_minimized_rights(&self) -> usize {
        self.0[Square::A1] as usize
            | ((self.0[Square::H1] as usize) << 1)
            | ((self.0[Square::A8] as usize) << 2)
            | ((self.0[Square::H8] as usize) << 3)
    }

    pub(crate) fn revoke(&mut self, square: Square) {
        self.0[square] = false;
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct Player {
    pub(crate) king: BitBoard,
    pub(crate) queens: BitBoard,
    pub(crate) rooks: BitBoard,
    pub(crate) bishops: BitBoard,
    pub(crate) knights: BitBoard,
    pub(crate) pawns: BitBoard,
    pub(crate) occupation: BitBoard,
    // All of the squares occupied by this player
    pub(crate) castling_rights: CastlingRights,
}

impl Player {
    pub(crate) fn blank() -> Self {
        Self {
            king: BitBoard::EMPTY,
            queens: BitBoard::EMPTY,
            rooks: BitBoard::EMPTY,
            bishops: BitBoard::EMPTY,
            knights: BitBoard::EMPTY,
            pawns: BitBoard::EMPTY,
            occupation: BitBoard::EMPTY,
            castling_rights: CastlingRights::empty(),
        }
    }

    fn piece_bitboard_mut(&mut self, kind: PieceKind) -> &mut BitBoard {
        match kind {
            PieceKind::King => &mut self.king,
            PieceKind::Queen => &mut self.queens,
            PieceKind::Rook => &mut self.rooks,
            PieceKind::Bishop => &mut self.bishops,
            PieceKind::Knight => &mut self.knights,
            PieceKind::Pawn => &mut self.pawns,
        }
    }

    pub(crate) fn piece_bitboard(&self, kind: PieceKind) -> BitBoard {
        match kind {
            PieceKind::King => self.king,
            PieceKind::Queen => self.queens,
            PieceKind::Rook => self.rooks,
            PieceKind::Bishop => self.bishops,
            PieceKind::Knight => self.knights,
            PieceKind::Pawn => self.pawns,
        }
    }

    pub(crate) fn toggle_piece(&mut self, square: Square, kind: PieceKind) {
        self.occupation.toggle_bit(square);
        self.piece_bitboard_mut(kind).toggle_bit(square);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct PieceTable([Option<PieceKind>; 64]);

impl PieceTable {
    pub(crate) fn piece_kind(&self, square: Square) -> Option<PieceKind> {
        self.0[square]
    }

    pub(crate) fn set(&mut self, square: Square, piece: Option<PieceKind>) {
        self.0[square] = piece;
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ColoredPieceTable([Option<Piece>; 64]);

impl ColoredPieceTable {
    pub(crate) const EMPTY: Self = Self([None; 64]);

    pub(crate) fn pieces(&self) -> &[Option<Piece>; 64] {
        &self.0
    }

    pub(crate) fn uncolored(&self) -> PieceTable {
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

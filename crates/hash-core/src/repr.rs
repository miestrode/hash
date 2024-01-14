use hash_bootstrap::{BitBoard, Color, ParseSquareError, Square};
use std::{
    fmt::{self, Display},
    ops::{Index, IndexMut},
    str::FromStr,
};

#[derive(Eq, Hash, Debug, Clone, Copy, PartialEq)]
/// Represents a type of piece, such as a [king](`PieceKind::King`),
/// or a [queen](`PieceKind::Queen`).
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl Display for PieceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PieceKind::King => 'k',
            PieceKind::Queen => 'q',
            PieceKind::Rook => 'r',
            PieceKind::Bishop => 'b',
            PieceKind::Knight => 'n',
            PieceKind::Pawn => 'p',
        }
        .fmt(f)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("piece must be a `k`, `q`, `r`, `b`, `n` or `p`")]
pub struct ParsePieceKindError;

impl FromStr for PieceKind {
    type Err = ParsePieceKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "k" => PieceKind::King,
            "q" => PieceKind::Queen,
            "r" => PieceKind::Rook,
            "b" => PieceKind::Bishop,
            "n" => PieceKind::Knight,
            "p" => PieceKind::Pawn,
            _ => return Err(ParsePieceKindError),
        })
    }
}

impl PieceKind {
    /// An array of each piece a pawn can promote to.
    pub const PROMOTIONS: [Self; 4] = [Self::Queen, Self::Rook, Self::Bishop, Self::Knight];
}

#[derive(Clone, Copy, PartialEq)]
/// Represents a Chess piece, which has a [type](`PieceKind`) and a [color](`Color`).
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
        let mut piece_char = match self.kind {
            PieceKind::King => 'k',
            PieceKind::Queen => 'q',
            PieceKind::Rook => 'r',
            PieceKind::Bishop => 'b',
            PieceKind::Knight => 'n',
            PieceKind::Pawn => 'p',
        };

        if self.color == Color::White {
            piece_char.make_ascii_uppercase()
        }

        piece_char.fmt(f)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("piece must be a `k`, `q`, `r`, `b`, `n` or `p`, case-insensitively")]
pub struct ParsePieceError;

impl TryFrom<char> for Piece {
    type Error = ParsePieceError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        Ok(match c {
            'k' => Piece::BLACK_KING,
            'q' => Piece::BLACK_QUEEN,
            'r' => Piece::BLACK_ROOK,
            'b' => Piece::BLACK_BISHOP,
            'n' => Piece::BLACK_KNIGHT,
            'p' => Piece::BLACK_PAWN,
            'K' => Piece::WHITE_KING,
            'Q' => Piece::WHITE_QUEEN,
            'R' => Piece::WHITE_ROOK,
            'B' => Piece::WHITE_BISHOP,
            'N' => Piece::WHITE_KNIGHT,
            'P' => Piece::WHITE_PAWN,
            _ => return Err(ParsePieceError),
        })
    }
}

#[derive(Eq, Clone, Copy, Debug)]
/// Represents a move in the game of Chess. To create a move one can use [`Board::interpret_move`].
pub struct ChessMove {
    pub origin: Square,
    pub target: Square,
    pub promotion: Option<PieceKind>,
}

impl PartialEq for ChessMove {
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin
            && self.target == other.target
            && self.promotion == other.promotion
    }
}

impl Display for ChessMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.origin.fmt(f)?;
        self.target.fmt(f)?;

        if let Some(kind) = self.promotion {
            kind.fmt(f)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum ParseChessMoveError {
    #[error("move must be 4 or 5 chars")]
    InvalidLength,
    #[error("invalid origin square")]
    InvalidOriginSquare(#[source] ParseSquareError),
    #[error("invalid target square")]
    InvalidTargetSquare(#[source] ParseSquareError),
    #[error("invalid promotion piece")]
    InvalidPromotionPiece(#[source] ParsePieceKindError),
}

impl FromStr for ChessMove {
    type Err = ParseChessMoveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 5 && s.len() != 4 {
            Err(ParseChessMoveError::InvalidLength)
        } else {
            let origin =
                Square::from_str(&s[0..2]).map_err(ParseChessMoveError::InvalidOriginSquare)?;
            let target =
                Square::from_str(&s[2..4]).map_err(ParseChessMoveError::InvalidTargetSquare)?;
            let promotion = if s.len() == 5 {
                Some(
                    PieceKind::from_str(&s[4..5])
                        .map_err(ParseChessMoveError::InvalidPromotionPiece)?,
                )
            } else {
                None
            };

            Ok(ChessMove {
                origin,
                target,
                promotion,
            })
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CastlingRights([bool; 64]);

impl CastlingRights {
    pub fn empty() -> Self {
        Self([false; 64])
    }

    pub fn can_castle_king_side(&self) -> bool {
        (self.0[Square::E1] ^ self.0[Square::E8]) && (self.0[Square::H1] ^ self.0[Square::H8])
    }

    pub fn can_castle_queen_side(&self) -> bool {
        (self.0[Square::E1] ^ self.0[Square::E8]) && (self.0[Square::A1] ^ self.0[Square::A8])
    }

    pub fn as_minimized_rights(&self) -> usize {
        self.0[Square::A1] as usize
            | ((self.0[Square::H1] as usize) << 1)
            | ((self.0[Square::A8] as usize) << 2)
            | ((self.0[Square::H8] as usize) << 3)
    }
}

impl Index<Square> for CastlingRights {
    type Output = bool;

    fn index(&self, index: Square) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<Square> for CastlingRights {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Player {
    pub king: BitBoard,
    pub queens: BitBoard,
    pub rooks: BitBoard,
    pub bishops: BitBoard,
    pub knights: BitBoard,
    pub pawns: BitBoard,
    pub occupation: BitBoard,
    // All of the squares occupied by this player
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

    pub fn piece_bitboard(&self, kind: PieceKind) -> BitBoard {
        match kind {
            PieceKind::King => self.king,
            PieceKind::Queen => self.queens,
            PieceKind::Rook => self.rooks,
            PieceKind::Bishop => self.bishops,
            PieceKind::Knight => self.knights,
            PieceKind::Pawn => self.pawns,
        }
    }

    pub fn toggle_piece(&mut self, square: Square, kind: PieceKind) {
        self.occupation.toggle_bit(square);
        self.piece_bitboard_mut(kind).toggle_bit(square);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PieceKindBoard([Option<PieceKind>; 64]);

impl PieceKindBoard {
    pub fn into_inner(self) -> [Option<PieceKind>; 64] {
        self.0
    }
}

impl Index<Square> for PieceKindBoard {
    type Output = Option<PieceKind>;

    fn index(&self, index: Square) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<Square> for PieceKindBoard {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[derive(Clone, Copy)]
pub struct PieceBoard([Option<Piece>; 64]);

impl PieceBoard {
    pub const EMPTY: Self = Self([None; 64]);

    pub fn new(pieces: [Option<Piece>; 64]) -> Self {
        Self(pieces)
    }

    pub fn uncolored(&self) -> PieceKindBoard {
        PieceKindBoard(
            self.0
                .into_iter()
                .map(|square| square.map(|piece| piece.kind))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    pub fn into_inner(self) -> [Option<Piece>; 64] {
        self.0
    }
}

impl Index<Square> for PieceBoard {
    type Output = Option<Piece>;

    fn index(&self, index: Square) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<Square> for PieceBoard {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum ParsePieceBoardError {
    #[error("board must have 8 rows")]
    InvalidRowLength,
    #[error("char in row must be a piece, or a digit from 1 to 8")]
    InvalidRowChar,
    #[error("spacing in row is invalid")]
    ColumnOffsetOverflow,
}

impl FromStr for PieceBoard {
    type Err = ParsePieceBoardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut board_pieces = PieceBoard::EMPTY;

        let rows = s.split('/').collect::<Vec<_>>();

        if rows.len() != 8 {
            return Err(ParsePieceBoardError::InvalidRowLength);
        }

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
                        board_pieces[Square::try_from(row_offset + column_offset as u8).unwrap()] =
                            Some(
                                Piece::try_from(character)
                                    .map_err(|_| ParsePieceBoardError::InvalidRowChar)?,
                            );
                    }
                }

                if column_offset > 7 {
                    return Err(ParsePieceBoardError::ColumnOffsetOverflow);
                }
            }
        }

        Ok(board_pieces)
    }
}

impl Display for PieceBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for column in (0..8).rev() {
            let mut spacing = 0;

            for row in 0..8 {
                let square = Square::try_from(column * 8 + row).unwrap();
                let piece = self[square];

                if let Some(piece) = piece {
                    if spacing != 0 {
                        spacing.fmt(f)?;
                    }

                    piece.fmt(f)?;

                    spacing = 0;
                } else {
                    spacing += 1;
                }
            }

            if spacing != 0 {
                spacing.fmt(f)?;
            }

            if column != 0 {
                '/'.fmt(f)?;
            }
        }

        Ok(())
    }
}

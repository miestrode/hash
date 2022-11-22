use before_build::{BitBoard, Square};

pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

pub(crate) enum Color {
    White,
    Black,
}

pub(crate) struct Piece {
    kind: PieceKind,
    color: Color,
}

pub enum Move {
    Simple {
        origin: Square,
        target: Square,
        is_double_push: bool,
    },
    EnPassant {
        origin: Square,
        target: Square,
    },
    Promotion {
        to: PieceKind,
    },
    Ks,
    Qs,
}

pub(crate) struct Player {
    king: BitBoard,
    queens: BitBoard,
    rooks: BitBoard,
    bishops: BitBoard,
    knights: BitBoard,
    pawns: BitBoard,
    can_ks: bool,
    can_qs: bool,
}

pub struct Board {
    current_player: Player,
    opposing_player: Player,
    playing_side: Color,
    ep_square: Option<Square>,
    duck: Option<Square>,
}

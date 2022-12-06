use before_build::{BitBoard, Square};

#[derive(Clone, Copy)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceKind {
    pub const PROMOTIONS: [Self; 4] = [Self::Queen, Self::Rook, Self::Bishop, Self::Knight];
}

#[derive(Clone, Copy)]
pub enum Color {
    White,
    Black,
}

#[derive(Clone, Copy)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
}

#[derive(Clone, Copy)]
pub enum Move {
    Simple {
        origin: Square,
        target: Square,
        is_double_push: bool,
    },
    EnPassant {
        origin: Square,
        target: Square, // Represents where the pawn will go to, not where the piece it captures is
                        // at
    },
    Promotion {
        origin: Square,
        target: Square,
        to: PieceKind,
    },
    CastleKs,
    CastleQs,
}

#[derive(Clone, Copy)]
pub struct Pins {
    pub horizontal: BitBoard,
    pub vertical: BitBoard,
    pub diagonal: BitBoard,
    pub anti_diagonal: BitBoard,
}

// TODO: Make sure all the of x_movement functions are cached
impl Pins {
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

#[derive(Clone, Copy)]
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
    pub can_castle_ks: bool,
    pub can_castle_qs: bool,
}

#[derive(Clone, Copy)]
pub struct EpData {
    pub capture_point: BitBoard,
    pub pawn: Square,
}

#[derive(Clone, Copy)]
pub struct Board {
    pub current_player: Player,
    pub opposing_player: Player,
    pub playing_side: Color,
    pub ep_data: Option<EpData>,
}

impl Board {}

use std::{
    fmt::{Display, Write},
    mem,
    str::FromStr,
};

use before_build::{BitBoard, Orientation, Square};

use crate::{index, mg};

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
pub enum Color {
    White,
    Black,
}

impl FromStr for Color {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            Err("Input must contain a single character")
        } else {
            match s.chars().next().unwrap() {
                'w' => Ok(Color::White),
                'b' => Ok(Color::Black),
                _ => Err("Input must be a 'w' or 'b'"),
            }
        }
    }
}

impl Color {
    pub fn as_orientation(&self) -> Orientation {
        match self {
            Self::White => Orientation::BottomToTop,
            Self::Black => Orientation::TopToBottom,
        }
    }
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

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum Move {
    Simple {
        origin: Square,
        target: Square,
        is_double_push: bool,
        moved_kind: PieceKind,
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

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::Simple { origin, target, .. } => f.write_fmt(format_args!("{origin}{target}")),
            Move::EnPassant { origin, target } => f.write_fmt(format_args!("{origin}{target}")),
            Move::Promotion { origin, target, to } => {
                f.write_fmt(format_args!("{origin}{target}{to}"))
            }
            Move::CastleKs => f.write_str("ks"),
            Move::CastleQs => f.write_str("qs"),
        }
    }
}

#[derive(Clone, Copy)]
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

impl Player {
    pub const BLANK: Self = Self {
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
        can_castle_ks: false,
        can_castle_qs: false,
    };

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
}

#[derive(Clone, Copy)]
pub struct EpData {
    pub capture_point: BitBoard,
    pub pawn: Square,
}

#[derive(Clone, Copy)]
pub struct PieceTable(pub [Option<PieceKind>; 64]);

impl PieceTable {
    pub fn move_piece(&mut self, origin: Square, target: Square) {
        self.0.swap(origin.as_index(), target.as_index());
        self.set(origin, None);
    }

    pub fn piece_kind(&self, square: Square) -> Option<PieceKind> {
        self.0[square]
    }

    pub fn set(&mut self, square: Square, kind: Option<PieceKind>) {
        self.0[square] = kind;
    }
}

#[derive(Clone, Copy)]
pub struct Board {
    pub current_player: Player,
    pub opposing_player: Player,
    pub orientation: Orientation,
    pub piece_table: PieceTable,
    pub ep_data: Option<EpData>,
}

impl Default for Board {
    fn default() -> Self {
        // Taken from https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation
        Self::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}

impl Board {
    pub fn perft(&self, depth: u32) -> u64 {
        let moves = mg::gen_moves(self);

        match depth {
            // At a depth of one we know all next moves will reach depth zero. Thus, we can know they are all leaves and add one each to the nodes searched.
            1 => moves.len() as u64,
            _ => moves
                .into_iter()
                .map(|chess_move| {
                    let mut board = *self;

                    // SAFETY: Move was generated by the legal move generator
                    unsafe { board.make_move_unchecked(chess_move) };

                    board.perft(depth - 1)
                })
                .sum(),
        }
    }

    pub fn split_perft(&self, depth: u32) -> Vec<(Move, u64)> {
        let moves = mg::gen_moves(self);

        match depth {
            // At a depth of one we know all next moves will reach depth zero. Thus, we can know they are all leaves and add one each to the nodes searched.
            1 => moves
                .into_iter()
                .map(|chess_move| (chess_move, 1))
                .collect(),
            _ => moves
                .into_iter()
                .map(|chess_move| {
                    (chess_move, {
                        let mut board = *self;

                        // SAFETY: Move was generated by the legal move generator
                        unsafe { board.make_move_unchecked(chess_move) };

                        board.perft(depth - 1)
                    })
                })
                .collect(),
        }
    }

    #[rustfmt::skip]
    fn update_slide_constraints(&mut self) {
        let diagonal_sliders = self.opposing_player.bishops + self.opposing_player.queens;
        let cross_sliders = self.opposing_player.rooks + self.opposing_player.queens;

        let king_square = self.current_player.king.first_one_as_square();
        let (up, right, down, left) = index::separated_rook_slides(king_square, self.opposing_player.occupation);
        let (up_left, up_right, down_right, down_left) = index::separated_bishop_slides(king_square, self.opposing_player.occupation);

        // NOTE: Macro is here for non-local control flow
        macro_rules! update {
            ($ray:expr, $possible_casters:expr, $pin_mask:expr) => {{
                // The ray stops when it finds an enemy slider, thus this check is sufficient
                if ($ray & $possible_casters).isnt_empty() {
                    let blockers = $ray & self.current_player.occupation;

                    if blockers.is_single_one() {
                        $pin_mask += $ray;
                    } else if blockers.is_empty() {
                        if self.current_player.valid_targets.is_full() {
                            self.current_player.valid_targets = $ray;
                        } else {
                            self.current_player.king_must_move = true;
                            return; // If the king must move then pin and check data are irrelevant
                        }
                    }
                }
            }};
        }

        update!(up,         cross_sliders,    self.current_player.pins.vertical);
        update!(up_right,   diagonal_sliders, self.current_player.pins.diagonal);
        update!(right,      cross_sliders,    self.current_player.pins.horizontal);
        update!(down_right, diagonal_sliders, self.current_player.pins.anti_diagonal);
        update!(down,       cross_sliders,    self.current_player.pins.vertical);
        update!(down_left,  diagonal_sliders, self.current_player.pins.diagonal);
        update!(left,       cross_sliders,    self.current_player.pins.horizontal);
        update!(up_left,    diagonal_sliders, self.current_player.pins.anti_diagonal);
    }

    fn update_non_slide_constraints(&mut self) {
        if self.current_player.king_must_move {
            return;
        }

        // The non-sliding attackers have to be either knights or pawns
        let attackers = (index::knight_attacks(self.current_player.king.first_one_as_square())
            & self.opposing_player.knights)
            + ((self.current_player.king.move_one_up_left(self.orientation)
                + self.current_player.king.move_one_up_right(self.orientation))
                & self.opposing_player.pawns);

        if attackers.is_single_one() {
            if self.current_player.valid_targets.is_full() {
                self.current_player.valid_targets = attackers;
            } else {
                self.current_player.king_must_move = true;
            }
        } else if attackers.isnt_empty() {
            // If this is true, there must be two attackers or more
            self.current_player.king_must_move = true;
        }
    }

    pub fn update_move_constraints(&mut self) {
        self.current_player.king_must_move = false;
        self.current_player.pins = Pins::EMPTY;
        self.current_player.valid_targets = BitBoard::FULL;

        mg::gen_dangers(self);

        self.update_slide_constraints();
        self.update_non_slide_constraints();
    }

    unsafe fn move_piece_unchecked(&mut self, kind: PieceKind, origin: Square, target: Square) {
        let captured_kind = self.piece_table.piece_kind(target);
        self.piece_table.move_piece(origin, target);

        // SAFETY: Data is assumed to be valid
        unsafe {
            self.current_player
                .move_piece_unchecked(kind, origin, target);
            if let Some(captured_kind) = captured_kind {
                self.opposing_player.toggle_piece(captured_kind, target);
            }
        }
    }

    // This function assumes the move at hand is actually properly constructed and legal
    pub unsafe fn make_move_unchecked(&mut self, chess_move: Move) {
        let past_ep_data = self.ep_data;
        self.ep_data = None;

        match chess_move {
            Move::Simple {
                origin,
                target,
                is_double_push,
                moved_kind,
            } => {
                if is_double_push {
                    self.ep_data = Some(EpData {
                        // SAFETY: Move is assumed to be properly constructed, and if it is a
                        // double push, this will always be in range
                        capture_point: unsafe { target.move_one_down_unchecked(self.orientation) }
                            .as_bitboard(),
                        pawn: target,
                    })
                }

                // Castling rights revocation checks
                {
                    if moved_kind == PieceKind::King {
                        self.current_player.can_castle_ks = false;
                        self.current_player.can_castle_qs = false;
                    }

                    let (
                        bottom_can_castle_ks,
                        bottom_can_castle_qs,
                        top_can_castle_ks,
                        top_can_castle_qs,
                    ) = match self.orientation {
                        Orientation::BottomToTop => (
                            &mut self.current_player.can_castle_ks,
                            &mut self.current_player.can_castle_qs,
                            &mut self.opposing_player.can_castle_ks,
                            &mut self.opposing_player.can_castle_qs,
                        ),
                        Orientation::TopToBottom => (
                            &mut self.opposing_player.can_castle_ks,
                            &mut self.opposing_player.can_castle_qs,
                            &mut self.current_player.can_castle_ks,
                            &mut self.current_player.can_castle_qs,
                        ),
                    };

                    if origin == Square::BOTTOM_LEFT_ROOK || target == Square::BOTTOM_LEFT_ROOK {
                        *bottom_can_castle_qs = false;
                    } else if origin == Square::BOTTOM_RIGHT_ROOK
                        || target == Square::BOTTOM_RIGHT_ROOK
                    {
                        *bottom_can_castle_ks = false;
                    }

                    if origin == Square::TOP_LEFT_ROOK || target == Square::TOP_LEFT_ROOK {
                        *top_can_castle_qs = false;
                    } else if origin == Square::TOP_RIGHT_ROOK || target == Square::TOP_RIGHT_ROOK {
                        *top_can_castle_ks = false;
                    }
                }

                // SAFETY: See above
                unsafe { self.move_piece_unchecked(moved_kind, origin, target) };
            }
            // SAFETY: Data is assumed to be valid and "EpData" is assumed to exist, as otherwise
            // en passant wouldn't make sense
            Move::EnPassant { origin, target } => unsafe {
                self.move_piece_unchecked(PieceKind::Pawn, origin, target);
                self.opposing_player
                    .toggle_piece(PieceKind::Pawn, past_ep_data.unwrap_unchecked().pawn);
            },
            Move::Promotion { origin, target, to } => {
                self.piece_table.set(origin, None);
                self.piece_table.set(target, Some(to));

                self.current_player.toggle_piece(PieceKind::Pawn, origin);
                self.current_player.toggle_piece(to, target);
            }
            Move::CastleKs => {
                // Based on https://en.wikipedia.org/wiki/Castling
                let (initial_king, end_king, initial_rook, end_rook) = match self.orientation {
                    Orientation::BottomToTop => (
                        Square::BOTTOM_KING,
                        Square::G1,
                        Square::BOTTOM_RIGHT_ROOK,
                        Square::F1,
                    ),
                    Orientation::TopToBottom => (
                        Square::TOP_KING,
                        Square::G8,
                        Square::TOP_RIGHT_ROOK,
                        Square::F8,
                    ),
                };

                // SAFETY: Moves are assumed to be correctly constructed and possible at this
                // position
                // TODO: Consider using a specialized function to avoid the capture checks that are
                // irrelevant if performance is improved
                unsafe {
                    self.move_piece_unchecked(PieceKind::King, initial_king, end_king);
                    self.move_piece_unchecked(PieceKind::Rook, initial_rook, end_rook);
                }

                self.current_player.can_castle_ks = false;
                self.current_player.can_castle_qs = false;
            }
            Move::CastleQs => {
                // Based on https://en.wikipedia.org/wiki/Castling
                let (initial_king, end_king, initial_rook, end_rook) = match self.orientation {
                    Orientation::BottomToTop => (
                        Square::BOTTOM_KING,
                        Square::C1,
                        Square::BOTTOM_RIGHT_ROOK,
                        Square::D1,
                    ),
                    Orientation::TopToBottom => (
                        Square::TOP_KING,
                        Square::C8,
                        Square::TOP_RIGHT_ROOK,
                        Square::D8,
                    ),
                };

                // SAFETY: Moves are assumed to be correctly constructed and possible at this
                // position
                // TODO: Consider using a specialized function to avoid the capture checks that are
                // irrelevant if performance is improved
                unsafe {
                    self.move_piece_unchecked(PieceKind::King, initial_king, end_king);
                    self.move_piece_unchecked(PieceKind::Rook, initial_rook, end_rook);
                }

                self.current_player.can_castle_ks = false;
                self.current_player.can_castle_qs = false;
            }
        }

        self.orientation = !self.orientation;
        mem::swap(&mut self.current_player, &mut self.opposing_player);
        self.update_move_constraints()
    }
}

struct ColoredPieceTable([Option<Piece>; 64]);

impl ColoredPieceTable {
    pub const EMPTY: Self = Self([None; 64]);
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

impl ColoredPieceTable {
    fn uncolored(&self) -> PieceTable {
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

impl FromStr for Board {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(' ').collect::<Vec<_>>();

        if parts.len() != 6 {
            Err("Input must contain 6 parts separated by spaces")
        } else {
            let colored_piece_table = ColoredPieceTable::from_str(parts[0])?;
            let color = Color::from_str(parts[1])?;
            let orientation = color.as_orientation();

            let ep_data = match parts[3] {
                "-" => None,
                square => Some({
                    let capture_point = Square::from_str(square)?.as_bitboard();
                    EpData {
                        capture_point,
                        pawn: capture_point.move_one_up(orientation).first_one_as_square(),
                    }
                }),
            };

            // These values currently aren't needed anywhere.
            if parts[4].parse::<u32>().is_err() {
                return Err("Input contains invalid number for half-moves");
            }
            if parts[5].parse::<u32>().is_err() {
                return Err("Input contains invalid number for full-moves");
            }

            let mut white = Player::BLANK;
            let mut black = Player::BLANK;

            for (square_index, piece) in colored_piece_table.0.into_iter().enumerate() {
                if let Some(Piece {
                    kind,
                    color: Color::White,
                }) = piece
                {
                    white.toggle_piece(kind, Square(square_index as u32));
                } else if let Some(Piece {
                    kind,
                    color: Color::Black,
                }) = piece
                {
                    black.toggle_piece(kind, Square(square_index as u32));
                }
            }

            if parts[2] != "-" {
                white.can_castle_ks = parts[2].contains('K');
                white.can_castle_qs = parts[2].contains('Q');
                black.can_castle_ks = parts[2].contains('k');
                black.can_castle_qs = parts[2].contains('q');

                // This would indicate the part contains some characters other than K, Q, k or q.
                if (white.can_castle_ks as usize
                    + white.can_castle_qs as usize
                    + black.can_castle_ks as usize
                    + black.can_castle_qs as usize)
                    != parts[2].len()
                {
                    return Err("Input contains invalid data for castling information");
                }
            }

            let (current_player, opposing_player) = match color {
                Color::White => (white, black),
                Color::Black => (black, white),
            };

            let mut board = Self {
                current_player,
                opposing_player,
                orientation,
                piece_table: colored_piece_table.uncolored(),
                ep_data,
            };

            board.update_move_constraints();

            Ok(board)
        }
    }
}

use hash_bootstrap::{BitBoard, Color, Square};

use crate::{
    cache::CacheHash,
    index,
    repr::{Move, Piece, PieceTable, Player},
};

#[derive(Clone, Copy)]
pub(crate) struct Board {
    pub(crate) us: Player,
    pub(crate) them: Player,
    pub(crate) checkers: BitBoard,
    pub(crate) pinned: BitBoard,
    pub(crate) playing_color: Color,
    pub(crate) en_passant_square: Option<Square>,
    piece_table: PieceTable,
    hash: u64,
}

impl CacheHash for Board {
    fn hash(&self) -> u64 {
        self.hash
    }
}

impl Board {
    pub(crate) fn is_attacked(&self, square: Square) -> bool {
        let mut attackers = BitBoard::EMPTY;

        attackers +=
            index::rook_slides(square, self.occupation()) & (self.them.rooks + self.them.queens);
        attackers += index::bishop_slides(square, self.occupation())
            & (self.them.bishops + self.them.queens);

        attackers += index::knight_attacks(square) & self.them.knights;
        attackers += index::king_attacks(square) & self.them.king;

        let square: BitBoard = square.into();

        attackers += (square.move_one_up_left(self.playing_color)
            + square.move_one_up_right(self.playing_color))
            & self.them.pawns;

        return !attackers.is_empty();
    }

    pub(crate) fn in_check(&self) -> bool {
        !self.checkers.is_empty()
    }

    pub(crate) fn occupation(&self) -> BitBoard {
        self.us.occupation + self.them.occupation
    }

    // NOTE: The function returns a boolean, such that it is true if the move was a pawn move or a
    // piece capture.
    pub(crate) unsafe fn make_move_unchecked(&mut self, chess_move: &Move) -> bool {
        self.playing_color = !self.playing_color;

        true
    }
}

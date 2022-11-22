#![feature(const_trait_impl, const_ops)]
mod bitboard;
mod square;

pub use {bitboard::*, square::*};

// All of the edges of the board. Useful for slide generation, since they represent areas that will
// be reached no matter if they are blocked by pieces, since blockers can be eaten
const EDGES: BitBoard = BitBoard::A_FILE + BitBoard::H_FILE + BitBoard::RANK_1 + BitBoard::RANK_8;

#[derive(Clone, Copy, Debug)]
pub struct Metadata {
    pub offset: usize,
    pub mask: BitBoard,
    #[cfg(not(target_feature = "bmi2"))]
    pub magic: u64,
}

// PERF: This is very slow, especially when used to spew out multiple rays, however, it doesn't
// matter since this isn't used to actually generate rays during runtime
pub fn gen_ray(
    pieces: BitBoard,
    blockers: BitBoard,
    update_fn: impl Fn(BitBoard) -> BitBoard,
) -> BitBoard {
    let mut rays = pieces;

    (loop {
        // Basically, you can at most go to positions occupied by blockers, not past them. Because
        // of this, ray positions with blockers in them are removed, so they won't be advanced
        let moveable_rays = rays - blockers;

        let next_rays = rays + update_fn(moveable_rays);

        if rays == next_rays {
            break rays;
        }

        rays = next_rays;
    }) - pieces
}

// This function returns the horizontal and vertical slide separately, as they are needed for
// correct "gen_cross_index" edges.
// NOTE: Blockers can be eaten
pub fn gen_cross_slides_separated(pieces: BitBoard, blockers: BitBoard) -> (BitBoard, BitBoard) {
    (
        gen_ray(pieces, blockers, |s| {
            BitBoard::move_one_up(s, Orientation::BottomToTop)
        }) + gen_ray(pieces, blockers, |s| {
            BitBoard::move_one_down(s, Orientation::BottomToTop)
        }),
        gen_ray(pieces, blockers, BitBoard::move_one_left)
            + gen_ray(pieces, blockers, BitBoard::move_one_right),
    )
}

// This function returns the horizontal and vertical slide separately, as they are needed for
// correct "gen_cross_index" edges.
// NOTE: Blockers can be eaten
pub fn gen_cross_slides(pieces: BitBoard, blockers: BitBoard) -> BitBoard {
    gen_ray(pieces, blockers, |s| {
        BitBoard::move_one_up(s, Orientation::BottomToTop)
    }) + gen_ray(pieces, blockers, |s| {
        BitBoard::move_one_down(s, Orientation::BottomToTop)
    }) + gen_ray(pieces, blockers, BitBoard::move_one_left)
        + gen_ray(pieces, blockers, BitBoard::move_one_right)
}

// NOTE: Blockers can be eaten
pub fn gen_diagonal_slides(pieces: BitBoard, blockers: BitBoard) -> BitBoard {
    gen_ray(pieces, blockers, |s| {
        BitBoard::move_one_up_right(s, Orientation::BottomToTop)
    }) + gen_ray(pieces, blockers, |s| {
        BitBoard::move_one_up_left(s, Orientation::BottomToTop)
    }) + gen_ray(pieces, blockers, |s| {
        BitBoard::move_one_down_left(s, Orientation::BottomToTop)
    }) + gen_ray(pieces, blockers, |s| {
        BitBoard::move_one_down_right(s, Orientation::BottomToTop)
    })
}

pub fn gen_knight_index(piece: BitBoard) -> BitBoard {
    let top = piece.move_one_up(Orientation::BottomToTop);
    let bottom = piece.move_one_down(Orientation::BottomToTop);
    let left = piece.move_one_left();
    let right = piece.move_one_right();

    top.move_one_up_right(Orientation::BottomToTop)
        + top.move_one_up_left(Orientation::BottomToTop)
        + left.move_one_up_left(Orientation::BottomToTop)
        + left.move_one_down_left(Orientation::BottomToTop)
        + bottom.move_one_down_left(Orientation::BottomToTop)
        + bottom.move_one_down_right(Orientation::BottomToTop)
        + right.move_one_up_right(Orientation::BottomToTop)
        + right.move_one_down_right(Orientation::BottomToTop)
}

pub fn gen_king_index(piece: BitBoard) -> BitBoard {
    let line = piece.move_one_left() + piece + piece.move_one_right();
    line.move_one_up(Orientation::BottomToTop) + line + line.move_one_down(Orientation::BottomToTop)
        - piece
}

pub fn gen_cross_mask(piece: BitBoard) -> BitBoard {
    let (v_slides, h_slides) = gen_cross_slides_separated(piece, BitBoard::EMPTY);
    let correct_edges = (BitBoard::A_FILE + BitBoard::H_FILE - v_slides)
        + (BitBoard::RANK_1 + BitBoard::RANK_8 - h_slides);

    // All slide collections blocked by pieces will be subsets of this template
    v_slides + h_slides - correct_edges
}

pub fn gen_diagonal_mask(piece: BitBoard) -> BitBoard {
    gen_diagonal_slides(piece, BitBoard::EMPTY) - EDGES
}

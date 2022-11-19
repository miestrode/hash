#![feature(const_trait_impl)]
use std::{env, fmt::Debug, fs, io::Error, path::PathBuf};

use before_build::{BitBoard, Orientation, Square};

// All of the edges of the board. Useful for slide generation, since they represent areas that will
// be reached no matter if they are blocked by pieces, since blockers can be eaten
const EDGES: BitBoard = BitBoard::A_FILE + BitBoard::H_FILE + BitBoard::RANK_1 + BitBoard::RANK_8;

// PERF: This is very slow, especially when used to spew out multiple rays, however, it doesn't
// matter since this isn't used to actually generate rays.
fn gen_ray(
    pieces: BitBoard,
    blockers: BitBoard,
    update_fn: impl Fn(BitBoard) -> BitBoard,
) -> BitBoard {
    let mut rays = pieces;

    (loop {
        // Basically, you can at most go to positions occupied by blockers, not past them. Because
        // of this, ray positions with blockers in them are removed, so they won't be advanced
        let moveable_rays = rays - blockers;

        let next_rays = rays + update_fn(moveable_rays) + update_fn(moveable_rays);

        if rays == next_rays {
            break rays;
        }

        rays = next_rays;
    }) - pieces
}

// NOTE: Blockers can be eaten
fn gen_cross_slides(pieces: BitBoard, blockers: BitBoard) -> BitBoard {
    gen_ray(pieces, blockers, |s| {
        BitBoard::move_one_up(s, Orientation::BottomToTop)
    }) + gen_ray(pieces, blockers, |s| {
        BitBoard::move_one_down(s, Orientation::BottomToTop)
    }) + gen_ray(pieces, blockers, BitBoard::move_one_left)
        + gen_ray(pieces, blockers, BitBoard::move_one_right)
}

// NOTE: Blockers can be eaten
fn gen_diagonal_slides(pieces: BitBoard, blockers: BitBoard) -> BitBoard {
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

// The function takes in a particular origin square for a rook-like piece, and then generates an
// association list of all the possible ways enemies may block that piece's path, with the resulting
// path (locations it can slide to)
fn gen_cross_index(piece: BitBoard) -> Vec<BitBoard> {
    // All slide collections blocked by pieces will be subsets of this template
    let template = gen_cross_slides(piece, BitBoard::EMPTY) - EDGES;

    // Subsets are ordered from empty set to improper subset (all empty to template bitboard)
    template.subsets().collect()
}

// Same deal as function above, but for bishops
fn gen_diagonal_index(piece: BitBoard) -> Vec<BitBoard> {
    // All slide collections blocked by pieces will be subsets of this template
    let template = gen_diagonal_slides(piece, BitBoard::EMPTY) - EDGES;

    // Subsets are ordered from empty set to improper subset (all empty to template bitboard)
    template.subsets().collect()
}

// The first returned value is the raw data of the slide table. In order to properly index into it
// however, the second returned value is needed. It contains a list of offsets for every single
// square whose data is in the table. These tell us when each square starts or ends in the table,
// which is neccessary, since squares are not equally spaced out.
fn gen_slide_table(index_fn: impl Fn(BitBoard) -> Vec<BitBoard>) -> (Vec<BitBoard>, [usize; 64]) {
    let mut offsets = [0; 64];
    let mut offset = 0;
    let mut raw_table = vec![];

    for square in Square::ALL {
        offsets[square] = offset;

        let mut index = index_fn(square.as_bitboard());
        offset += index.len();
        raw_table.append(&mut index);
    }

    (raw_table, offsets)
}

fn gen_knight_index(piece: BitBoard) -> BitBoard {
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

fn gen_king_index(piece: BitBoard) -> BitBoard {
    let line = piece.move_one_left() + piece + piece.move_one_right();
    line.move_one_up(Orientation::BottomToTop) + line + line.move_one_down(Orientation::BottomToTop)
        - piece
}

fn gen_piece_table(move_fn: impl Fn(BitBoard) -> BitBoard) -> Vec<BitBoard> {
    Square::ALL
        .into_iter()
        .map(|square| move_fn(square.as_bitboard()))
        .collect()
}

fn stringify_table<T: Debug>(name: &'static str, data_type: &'static str, data: &[T]) -> String {
    let mut output = format!("const {name}: [{data_type}; {}] = [", data.len());

    for element in data {
        output.push_str(&format!("{element:?},"))
    }

    output.push_str("];");
    output
}

fn main() -> Result<(), Error> {
    let output_file = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("table.rs");

    let (cross_data, cross_offsets) = gen_slide_table(gen_cross_index);
    let (diagonal_data, diagonal_offsets) = gen_slide_table(gen_diagonal_index);

    fs::write(
        &output_file,
        stringify_table("CROSS_SLIDES", "BitBoard", &cross_data)
            + &stringify_table("CROSS_OFFSETS", "usize", &cross_offsets)
            + &stringify_table("DIAGONAL_SLIDES", "BitBoard", &diagonal_data)
            + &stringify_table("DIAGONAL_OFFSETS", "usize", &diagonal_offsets)
            + &stringify_table(
                "KNIGHT_MOVES",
                "BitBoard",
                &gen_piece_table(gen_knight_index),
            )
            + &stringify_table("KING_MOVES", "BitBoard", &gen_piece_table(gen_king_index)),
    )?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

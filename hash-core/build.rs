use std::{array, io::Error};

use rand::{rngs::StdRng, Rng, SeedableRng};

use hash_bootstrap::{BitBoard, Color, Metadata, Square, ZobristMap};
use rustifact::ToTokenStream;

const SEED: u64 = 0x73130172E6DEA605;

fn gen_ray(
    pieces: BitBoard,
    blockers: BitBoard,
    update_fn: impl Fn(BitBoard) -> BitBoard,
) -> BitBoard {
    let mut rays = pieces;

    (loop {
        // Basically, you can at most go to positions occupied by blockers, not past them. Because
        // of this, ray positions with blockers in them are removed, so they won't be advanced
        let moveable_rays = rays & !blockers;

        let next_rays = rays | update_fn(moveable_rays);

        if rays == next_rays {
            break rays;
        }

        rays = next_rays;
    }) & !pieces
}

fn gen_separated_cross_slides(
    pieces: BitBoard,
    blockers: BitBoard,
) -> (BitBoard, BitBoard, BitBoard, BitBoard) {
    (
        gen_ray(pieces, blockers, |state| {
            BitBoard::move_one_up(state, Color::White)
        }),
        gen_ray(pieces, blockers, |state| {
            BitBoard::move_one_right(state, Color::White)
        }),
        gen_ray(pieces, blockers, |state| {
            BitBoard::move_one_down(state, Color::White)
        }),
        gen_ray(pieces, blockers, |state| {
            BitBoard::move_one_left(state, Color::White)
        }),
    )
}

fn gen_rook_slides(pieces: BitBoard, blockers: BitBoard) -> BitBoard {
    let (up, right, down, left) = gen_separated_cross_slides(pieces, blockers);

    up | right | down | left
}

fn gen_bishop_slides(pieces: BitBoard, blockers: BitBoard) -> BitBoard {
    fn gen_separated_diagonal_slides(
        pieces: BitBoard,
        blockers: BitBoard,
    ) -> (BitBoard, BitBoard, BitBoard, BitBoard) {
        (
            gen_ray(pieces, blockers, |state| {
                BitBoard::move_one_up_left(state, Color::White)
            }),
            gen_ray(pieces, blockers, |state| {
                BitBoard::move_one_up_right(state, Color::White)
            }),
            gen_ray(pieces, blockers, |state| {
                BitBoard::move_one_down_right(state, Color::White)
            }),
            gen_ray(pieces, blockers, |state| {
                BitBoard::move_one_down_left(state, Color::White)
            }),
        )
    }

    let (up_left, up_right, down_right, down_left) =
        gen_separated_diagonal_slides(pieces, blockers);

    up_left | up_right | down_right | down_left
}

fn gen_knight_index(piece: BitBoard) -> BitBoard {
    let top = piece.move_one_up(Color::White);
    let bottom = piece.move_one_down(Color::White);
    let left = piece.move_one_left(Color::White);
    let right = piece.move_one_right(Color::White);

    top.move_one_up_right(Color::White)
        | top.move_one_up_left(Color::White)
        | left.move_one_up_left(Color::White)
        | left.move_one_down_left(Color::White)
        | bottom.move_one_down_left(Color::White)
        | bottom.move_one_down_right(Color::White)
        | right.move_one_up_right(Color::White)
        | right.move_one_down_right(Color::White)
}

fn gen_king_index(piece: BitBoard) -> BitBoard {
    let line = piece.move_one_left(Color::White) | piece | piece.move_one_right(Color::White);
    (line.move_one_up(Color::White) | line | line.move_one_down(Color::White)) & !piece
}

fn gen_rook_mask(piece: BitBoard) -> BitBoard {
    let (up, right, down, left) = gen_separated_cross_slides(piece, BitBoard::EMPTY);
    let correct_edges =
        (BitBoard::EDGE_FILES & !(up | down)) | (BitBoard::EDGE_RANKS & !(left | right));

    // All slide collections blocked by pieces will be subsets of this template
    (up | right | down | left) & !correct_edges
}

fn gen_bishop_mask(piece: BitBoard) -> BitBoard {
    gen_bishop_slides(piece, BitBoard::EMPTY) & !BitBoard::EDGES
}

fn gen_piece_table(move_fn: impl Fn(BitBoard) -> BitBoard) -> Vec<BitBoard> {
    Square::ALL
        .into_iter()
        .map(|square| move_fn(square.into()))
        .collect()
}

fn try_metadata(
    square: Square,
    slide_fn: impl Fn(BitBoard, BitBoard) -> BitBoard,
    metadata: Metadata,
) -> Option<Vec<BitBoard>> {
    let mut slides = vec![BitBoard::EMPTY; 1 << (64 - metadata.shift)];

    for subset in metadata.mask.subsets() {
        let index = metadata.create_local_index(subset);

        if slides[index].is_empty() {
            slides[index] = slide_fn(square.into(), subset);
        } else {
            return None;
        }
    }

    Some(slides)
}

fn find_metadata(
    square: Square,
    mask_fn: &impl Fn(BitBoard) -> BitBoard,
    slide_fn: &impl Fn(BitBoard, BitBoard) -> BitBoard,
    offset: usize,
) -> (Metadata, Vec<BitBoard>) {
    let mut rng = StdRng::seed_from_u64(SEED);

    loop {
        let [magic_1, magic_2, magic_3]: [u64; 3] = rng.gen();
        let mask = mask_fn(square.into());

        let metadata = Metadata {
            offset,
            mask,
            magic: magic_1 & magic_2 & magic_3,
            shift: 64 - mask.count_ones() as usize,
        };

        if let Some(slides) = try_metadata(square, slide_fn, metadata) {
            break (metadata, slides);
        }
    }
}

fn generate_slides(
    mask_fn: &impl Fn(BitBoard) -> BitBoard,
    slide_fn: &impl Fn(BitBoard, BitBoard) -> BitBoard,
) -> (Vec<Metadata>, Vec<BitBoard>) {
    let mut metadatas = Vec::with_capacity(64);
    let mut slides = Vec::new();

    for square in Square::ALL {
        let (metadata, mut square_slides) = find_metadata(square, mask_fn, slide_fn, slides.len());

        slides.append(&mut square_slides);
        metadatas.push(metadata);
    }

    (metadatas, slides)
}

fn main() -> Result<(), Error> {
    let (rook_metadata, rook_slides) = generate_slides(&gen_rook_mask, &gen_rook_slides);

    rustifact::write_const_array!(ROOK_SLIDE_METADATA, Metadata, &rook_metadata);
    rustifact::write_const_array!(ROOK_SLIDES, BitBoard, &rook_slides);

    let (bishop_metadata, bishop_slides) = generate_slides(&gen_bishop_mask, &gen_bishop_slides);

    rustifact::write_const_array!(BISHOP_SLIDE_METADATA, Metadata, &bishop_metadata);
    rustifact::write_const_array!(BISHOP_SLIDES, BitBoard, &bishop_slides);

    rustifact::write_const_array!(KNIGHT_ATTACKS, BitBoard, &gen_piece_table(gen_knight_index));

    rustifact::write_const_array!(KING_ATTACKS, BitBoard, &gen_piece_table(gen_king_index));

    rustifact::write_const_array!(
        WHITE_PAWN_ATTACKS,
        BitBoard,
        &array::from_fn::<_, 64, _>(|index| {
            let square: BitBoard = Square::try_from(index as u8).unwrap().into();

            square.move_one_up_left(Color::White) | square.move_one_up_right(Color::White)
        })
    );

    rustifact::write_const_array!(
        BLACK_PAWN_ATTACKS,
        BitBoard,
        &array::from_fn::<_, 64, _>(|index| {
            let square: BitBoard = Square::try_from(index as u8).unwrap().into();

            square.move_one_up_left(Color::Black) | square.move_one_up_right(Color::Black)
        })
    );

    rustifact::write_const_array!(
        WHITE_PAWN_PUSHES,
        BitBoard : 1,
        &array::from_fn::<_, 64, _>(|index| {
            let square = Square::try_from(index as u8).unwrap();
            let square_as_bitboard: BitBoard = square.into();

            square_as_bitboard.move_one_up(Color::White)
                | if square.rank() == 1 {
                square_as_bitboard.move_two_up(Color::White)
            } else {
                BitBoard::EMPTY
            }
        })
    );

    rustifact::write_const_array!(
        BLACK_PAWN_PUSHES,
        BitBoard,
        &array::from_fn::<_, 64, _>(|index| {
            let square = Square::try_from(index as u8).unwrap();
            let square_as_bitboard: BitBoard = square.into();

            square_as_bitboard.move_one_up(Color::Black)
                | if square.rank() == 6 {
                    square_as_bitboard.move_two_up(Color::Black)
                } else {
                    BitBoard::EMPTY
                }
        })
    );

    rustifact::write_const_array!(
        LINE,
        BitBoard,
        &array::from_fn::<_, { 64 * 64 }, _>(|index| {
            let first_square = Square::try_from((index / 64) as u8).unwrap();
            let second_square = Square::try_from((index % 64) as u8).unwrap();

            Square::ALL
                .into_iter()
                .filter(|&square| square.on_line(first_square, second_square))
                .map(BitBoard::from)
                .fold(BitBoard::EMPTY, |board, square| board | square)
        })
    );

    rustifact::write_const_array!(
        BETWEEN,
        BitBoard,
        &array::from_fn::<_, { 64 * 64 }, _>(|index| {
            let first_square = Square::try_from((index / 64) as u8).unwrap();
            let second_square = Square::try_from((index % 64) as u8).unwrap();

            Square::ALL
                .into_iter()
                .filter(|&square| {
                    square.on_line(first_square, second_square)
                        && square.in_rectangle(first_square, second_square)
                        && (square != first_square)
                        && (square != second_square)
                })
                .fold(BitBoard::EMPTY, |board, square| board | square.into())
        })
    );

    rustifact::write_const!(
        ZOBRIST_MAP,
        ZobristMap,
        StdRng::seed_from_u64(SEED).gen::<ZobristMap>()
    );

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

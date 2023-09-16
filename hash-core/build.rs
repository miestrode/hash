use std::fs::{File, OpenOptions};
use std::io::Write;
use std::{array, env, fmt::Debug, io, io::Error, path::PathBuf};

use hash_bootstrap::{bb, BitBoard, Color, Square, ZobristMap};
use rand::{rngs::StdRng, Rng, SeedableRng};

const SEED: u64 = 0x73130172E6DEA605;

// Used for updating the structure based on build flags.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
struct Metadata {
    offset: usize,
    mask: BitBoard,
    #[cfg(not(target_feature = "bmi2"))]
    pub magic: u64,
}

// All of the edges of the board. Useful for slide generation, since they represent areas that will
// be reached no matter if they are blocked by pieces, since blockers can be eaten
const EDGES: BitBoard = bb!(
    0b11111111
    0b10000001
    0b10000001
    0b10000001
    0b10000001
    0b10000001
    0b10000001
    0b11111111
);

// PERF: This is very slow, especially when used to spew out multiple rays, however, it doesn't
// matter since this isn't used to actually generate rays during runtime
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

        let next_rays = rays + update_fn(moveable_rays);

        if rays == next_rays {
            break rays;
        }

        rays = next_rays;
    }) - pieces
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

fn gen_cross_slides(pieces: BitBoard, blockers: BitBoard) -> BitBoard {
    let (up, right, down, left) = gen_separated_cross_slides(pieces, blockers);

    up + right + down + left
}

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

fn gen_diagonal_slides(pieces: BitBoard, blockers: BitBoard) -> BitBoard {
    let (up_left, up_right, down_right, down_left) =
        gen_separated_diagonal_slides(pieces, blockers);
    up_left + up_right + down_right + down_left
}

fn gen_knight_index(piece: BitBoard) -> BitBoard {
    let top = piece.move_one_up(Color::White);
    let bottom = piece.move_one_down(Color::White);
    let left = piece.move_one_left(Color::White);
    let right = piece.move_one_right(Color::White);

    top.move_one_up_right(Color::White)
        + top.move_one_up_left(Color::White)
        + left.move_one_up_left(Color::White)
        + left.move_one_down_left(Color::White)
        + bottom.move_one_down_left(Color::White)
        + bottom.move_one_down_right(Color::White)
        + right.move_one_up_right(Color::White)
        + right.move_one_down_right(Color::White)
}

fn gen_king_index(piece: BitBoard) -> BitBoard {
    let line = piece.move_one_left(Color::White) + piece + piece.move_one_right(Color::White);
    line.move_one_up(Color::White) + line + line.move_one_down(Color::White) - piece
}

fn gen_cross_mask(piece: BitBoard) -> BitBoard {
    let (up, right, down, left) = gen_separated_cross_slides(piece, BitBoard::EMPTY);
    let correct_edges =
        (BitBoard::EDGE_FILES - (up + down)) + (BitBoard::EDGE_RANKS - (left + right));

    // All slide collections blocked by pieces will be subsets of this template
    up + right + down + left - correct_edges
}

fn gen_diagonal_mask(piece: BitBoard) -> BitBoard {
    gen_diagonal_slides(piece, BitBoard::EMPTY) - EDGES
}

fn gen_piece_table(move_fn: impl Fn(BitBoard) -> BitBoard) -> Vec<BitBoard> {
    Square::ALL
        .into_iter()
        .map(|square| move_fn(square.into()))
        .collect()
}

fn write_table<T: Debug>(name: &'static str, data: &[T], type_name: &'static str, file: &mut File) -> io::Result<()> {
    write!(
        file,
        "static {name}: [{type_name}; {}] = [",
        data.len()
    )?;

    for element in data {
        write!(file, "{element:?},")?;
    }

    write!(file, "];")
}

fn write_variable<T: Debug>(name: &'static str, data: T, type_name: &'static str, file: &mut File) -> io::Result<()> {
    write!(file, "static {name}: {type_name} = {data:?};")
}

fn main() -> Result<(), Error> {
    let mut output_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("out.rs"))?;

    #[cfg(target_feature = "bmi2")]
    {
        // The first returned value is the raw data of the slide table. In order to properly index into it
        // however, the second returned value is needed. It contains a list of offsets for every single
        // square whose data is in the table. These tell us when each square starts or ends in the table,
        // which is necessary, since squares are not equally spaced out.
        fn gen_slide_table(
            mask_fn: impl Fn(BitBoard) -> BitBoard,
            move_fn: impl Fn(BitBoard, BitBoard) -> BitBoard,
        ) -> (Vec<BitBoard>, [Metadata; 64]) {
            let mut metadata = [Metadata {
                mask: BitBoard::EMPTY,
                offset: 0,
            }; 64];
            let mut offset = 0;
            let mut raw_table = vec![];

            for square in Square::ALL {
                let template = mask_fn(square.into());

                metadata[square] = Metadata {
                    offset,
                    mask: template,
                };

                let mut index = template
                    .subsets()
                    .map(|subset| move_fn(square.into(), subset))
                    .collect::<Vec<_>>();

                offset += index.len();
                raw_table.append(&mut index);
            }

            (raw_table, metadata)
        }

        let (cross_data, cross_meta) = gen_slide_table(gen_cross_mask, gen_cross_slides);
        let (diagonal_data, diagonal_meta) =
            gen_slide_table(gen_diagonal_mask, gen_diagonal_slides);

        write_table("CROSS_SLIDES", &cross_data, "BitBoard", &mut output_file)?;
        write_table("CROSS_META", &cross_meta, "Metadata", &mut output_file)?;

        write_table("DIAGONAL_SLIDES", &diagonal_data, "BitBoard",  &mut output_file)?;
        write_table("DIAGONAL_META", &diagonal_meta, "Metadata", &mut output_file)?;
    }

    #[cfg(not(target_feature = "bmi2"))]
    {
        #[rustfmt::skip]
        let cross_metadata = [
            Metadata { mask: BitBoard(0x000101010101017e), magic: 0x00280077ffebfffe, offset: 26304 },
            Metadata { mask: BitBoard(0x000202020202027c), magic: 0x2004010201097fff, offset: 35520 },
            Metadata { mask: BitBoard(0x000404040404047a), magic: 0x0010020010053fff, offset: 38592 },
            Metadata { mask: BitBoard(0x0008080808080876), magic: 0x0040040008004002, offset:  8026 },
            Metadata { mask: BitBoard(0x001010101010106e), magic: 0x7fd00441ffffd003, offset: 22196 },
            Metadata { mask: BitBoard(0x002020202020205e), magic: 0x4020008887dffffe, offset: 80870 },
            Metadata { mask: BitBoard(0x004040404040403e), magic: 0x004000888847ffff, offset: 76747 },
            Metadata { mask: BitBoard(0x008080808080807e), magic: 0x006800fbff75fffd, offset: 30400 },
            Metadata { mask: BitBoard(0x0001010101017e00), magic: 0x000028010113ffff, offset: 11115 },
            Metadata { mask: BitBoard(0x0002020202027c00), magic: 0x0020040201fcffff, offset: 18205 },
            Metadata { mask: BitBoard(0x0004040404047a00), magic: 0x007fe80042ffffe8, offset: 53577 },
            Metadata { mask: BitBoard(0x0008080808087600), magic: 0x00001800217fffe8, offset: 62724 },
            Metadata { mask: BitBoard(0x0010101010106e00), magic: 0x00001800073fffe8, offset: 34282 },
            Metadata { mask: BitBoard(0x0020202020205e00), magic: 0x00001800e05fffe8, offset: 29196 },
            Metadata { mask: BitBoard(0x0040404040403e00), magic: 0x00001800602fffe8, offset: 23806 },
            Metadata { mask: BitBoard(0x0080808080807e00), magic: 0x000030002fffffa0, offset: 49481 },
            Metadata { mask: BitBoard(0x00010101017e0100), magic: 0x00300018010bffff, offset:  2410 },
            Metadata { mask: BitBoard(0x00020202027c0200), magic: 0x0003000c0085fffb, offset: 36498 },
            Metadata { mask: BitBoard(0x00040404047a0400), magic: 0x0004000802010008, offset: 24478 },
            Metadata { mask: BitBoard(0x0008080808760800), magic: 0x0004002020020004, offset: 10074 },
            Metadata { mask: BitBoard(0x00101010106e1000), magic: 0x0001002002002001, offset: 79315 },
            Metadata { mask: BitBoard(0x00202020205e2000), magic: 0x0001001000801040, offset: 51779 },
            Metadata { mask: BitBoard(0x00404040403e4000), magic: 0x0000004040008001, offset: 13586 },
            Metadata { mask: BitBoard(0x00808080807e8000), magic: 0x0000006800cdfff4, offset: 19323 },
            Metadata { mask: BitBoard(0x000101017e010100), magic: 0x0040200010080010, offset: 70612 },
            Metadata { mask: BitBoard(0x000202027c020200), magic: 0x0000080010040010, offset: 83652 },
            Metadata { mask: BitBoard(0x000404047a040400), magic: 0x0004010008020008, offset: 63110 },
            Metadata { mask: BitBoard(0x0008080876080800), magic: 0x0000040020200200, offset: 34496 },
            Metadata { mask: BitBoard(0x001010106e101000), magic: 0x0002008010100100, offset: 84966 },
            Metadata { mask: BitBoard(0x002020205e202000), magic: 0x0000008020010020, offset: 54341 },
            Metadata { mask: BitBoard(0x004040403e404000), magic: 0x0000008020200040, offset: 60421 },
            Metadata { mask: BitBoard(0x008080807e808000), magic: 0x0000820020004020, offset: 86402 },
            Metadata { mask: BitBoard(0x0001017e01010100), magic: 0x00fffd1800300030, offset: 50245 },
            Metadata { mask: BitBoard(0x0002027c02020200), magic: 0x007fff7fbfd40020, offset: 76622 },
            Metadata { mask: BitBoard(0x0004047a04040400), magic: 0x003fffbd00180018, offset: 84676 },
            Metadata { mask: BitBoard(0x0008087608080800), magic: 0x001fffde80180018, offset: 78757 },
            Metadata { mask: BitBoard(0x0010106e10101000), magic: 0x000fffe0bfe80018, offset: 37346 },
            Metadata { mask: BitBoard(0x0020205e20202000), magic: 0x0001000080202001, offset:   370 },
            Metadata { mask: BitBoard(0x0040403e40404000), magic: 0x0003fffbff980180, offset: 42182 },
            Metadata { mask: BitBoard(0x0080807e80808000), magic: 0x0001fffdff9000e0, offset: 45385 },
            Metadata { mask: BitBoard(0x00017e0101010100), magic: 0x00fffefeebffd800, offset: 61659 },
            Metadata { mask: BitBoard(0x00027c0202020200), magic: 0x007ffff7ffc01400, offset: 12790 },
            Metadata { mask: BitBoard(0x00047a0404040400), magic: 0x003fffbfe4ffe800, offset: 16762 },
            Metadata { mask: BitBoard(0x0008760808080800), magic: 0x001ffff01fc03000, offset:     0 },
            Metadata { mask: BitBoard(0x00106e1010101000), magic: 0x000fffe7f8bfe800, offset: 38380 },
            Metadata { mask: BitBoard(0x00205e2020202000), magic: 0x0007ffdfdf3ff808, offset: 11098 },
            Metadata { mask: BitBoard(0x00403e4040404000), magic: 0x0003fff85fffa804, offset: 21803 },
            Metadata { mask: BitBoard(0x00807e8080808000), magic: 0x0001fffd75ffa802, offset: 39189 },
            Metadata { mask: BitBoard(0x007e010101010100), magic: 0x00ffffd7ffebffd8, offset: 58628 },
            Metadata { mask: BitBoard(0x007c020202020200), magic: 0x007fff75ff7fbfd8, offset: 44116 },
            Metadata { mask: BitBoard(0x007a040404040400), magic: 0x003fff863fbf7fd8, offset: 78357 },
            Metadata { mask: BitBoard(0x0076080808080800), magic: 0x001fffbfdfd7ffd8, offset: 44481 },
            Metadata { mask: BitBoard(0x006e101010101000), magic: 0x000ffff810280028, offset: 64134 },
            Metadata { mask: BitBoard(0x005e202020202000), magic: 0x0007ffd7f7feffd8, offset: 41759 },
            Metadata { mask: BitBoard(0x003e404040404000), magic: 0x0003fffc0c480048, offset:  1394 },
            Metadata { mask: BitBoard(0x007e808080808000), magic: 0x0001ffffafd7ffd8, offset: 40910 },
            Metadata { mask: BitBoard(0x7e01010101010100), magic: 0x00ffffe4ffdfa3ba, offset: 66516 },
            Metadata { mask: BitBoard(0x7c02020202020200), magic: 0x007fffef7ff3d3da, offset:  3897 },
            Metadata { mask: BitBoard(0x7a04040404040400), magic: 0x003fffbfdfeff7fa, offset:  3930 },
            Metadata { mask: BitBoard(0x7608080808080800), magic: 0x001fffeff7fbfc22, offset: 72934 },
            Metadata { mask: BitBoard(0x6e10101010101000), magic: 0x0000020408001001, offset: 72662 },
            Metadata { mask: BitBoard(0x5e20202020202000), magic: 0x0007fffeffff77fd, offset: 56325 },
            Metadata { mask: BitBoard(0x3e40404040404000), magic: 0x0003ffffbf7dfeec, offset: 66501 },
            Metadata { mask: BitBoard(0x7e80808080808000), magic: 0x0001ffff9dffa333, offset: 14826 },
        ];

        #[rustfmt::skip]
        let diagonal_metadata = [
            Metadata { mask: BitBoard(0x0040201008040200), magic: 0x007fbfbfbfbfbfff, offset:  5378 },
            Metadata { mask: BitBoard(0x0000402010080400), magic: 0x0000a060401007fc, offset:  4093 },
            Metadata { mask: BitBoard(0x0000004020100a00), magic: 0x0001004008020000, offset:  4314 },
            Metadata { mask: BitBoard(0x0000000040221400), magic: 0x0000806004000000, offset:  6587 },
            Metadata { mask: BitBoard(0x0000000002442800), magic: 0x0000100400000000, offset:  6491 },
            Metadata { mask: BitBoard(0x0000000204085000), magic: 0x000021c100b20000, offset:  6330 },
            Metadata { mask: BitBoard(0x0000020408102000), magic: 0x0000040041008000, offset:  5609 },
            Metadata { mask: BitBoard(0x0002040810204000), magic: 0x00000fb0203fff80, offset: 22236 },
            Metadata { mask: BitBoard(0x0020100804020000), magic: 0x0000040100401004, offset:  6106 },
            Metadata { mask: BitBoard(0x0040201008040000), magic: 0x0000020080200802, offset:  5625 },
            Metadata { mask: BitBoard(0x00004020100a0000), magic: 0x0000004010202000, offset: 16785 },
            Metadata { mask: BitBoard(0x0000004022140000), magic: 0x0000008060040000, offset: 16817 },
            Metadata { mask: BitBoard(0x0000000244280000), magic: 0x0000004402000000, offset:  6842 },
            Metadata { mask: BitBoard(0x0000020408500000), magic: 0x0000000801008000, offset:  7003 },
            Metadata { mask: BitBoard(0x0002040810200000), magic: 0x000007efe0bfff80, offset:  4197 },
            Metadata { mask: BitBoard(0x0004081020400000), magic: 0x0000000820820020, offset:  7356 },
            Metadata { mask: BitBoard(0x0010080402000200), magic: 0x0000400080808080, offset:  4602 },
            Metadata { mask: BitBoard(0x0020100804000400), magic: 0x00021f0100400808, offset:  4538 },
            Metadata { mask: BitBoard(0x004020100a000a00), magic: 0x00018000c06f3fff, offset: 29531 },
            Metadata { mask: BitBoard(0x0000402214001400), magic: 0x0000258200801000, offset: 45393 },
            Metadata { mask: BitBoard(0x0000024428002800), magic: 0x0000240080840000, offset: 12420 },
            Metadata { mask: BitBoard(0x0002040850005000), magic: 0x000018000c03fff8, offset: 15763 },
            Metadata { mask: BitBoard(0x0004081020002000), magic: 0x00000a5840208020, offset:  5050 },
            Metadata { mask: BitBoard(0x0008102040004000), magic: 0x0000020008208020, offset:  4346 },
            Metadata { mask: BitBoard(0x0008040200020400), magic: 0x0000804000810100, offset:  6074 },
            Metadata { mask: BitBoard(0x0010080400040800), magic: 0x0001011900802008, offset:  7866 },
            Metadata { mask: BitBoard(0x0020100a000a1000), magic: 0x0000804000810100, offset: 32139 },
            Metadata { mask: BitBoard(0x0040221400142200), magic: 0x000100403c0403ff, offset: 57673 },
            Metadata { mask: BitBoard(0x0002442800284400), magic: 0x00078402a8802000, offset: 55365 },
            Metadata { mask: BitBoard(0x0004085000500800), magic: 0x0000101000804400, offset: 15818 },
            Metadata { mask: BitBoard(0x0008102000201000), magic: 0x0000080800104100, offset:  5562 },
            Metadata { mask: BitBoard(0x0010204000402000), magic: 0x00004004c0082008, offset:  6390 },
            Metadata { mask: BitBoard(0x0004020002040800), magic: 0x0001010120008020, offset:  7930 },
            Metadata { mask: BitBoard(0x0008040004081000), magic: 0x000080809a004010, offset: 13329 },
            Metadata { mask: BitBoard(0x00100a000a102000), magic: 0x0007fefe08810010, offset:  7170 },
            Metadata { mask: BitBoard(0x0022140014224000), magic: 0x0003ff0f833fc080, offset: 27267 },
            Metadata { mask: BitBoard(0x0044280028440200), magic: 0x007fe08019003042, offset: 53787 },
            Metadata { mask: BitBoard(0x0008500050080400), magic: 0x003fffefea003000, offset:  5097 },
            Metadata { mask: BitBoard(0x0010200020100800), magic: 0x0000101010002080, offset:  6643 },
            Metadata { mask: BitBoard(0x0020400040201000), magic: 0x0000802005080804, offset:  6138 },
            Metadata { mask: BitBoard(0x0002000204081000), magic: 0x0000808080a80040, offset:  7418 },
            Metadata { mask: BitBoard(0x0004000408102000), magic: 0x0000104100200040, offset:  7898 },
            Metadata { mask: BitBoard(0x000a000a10204000), magic: 0x0003ffdf7f833fc0, offset: 42012 },
            Metadata { mask: BitBoard(0x0014001422400000), magic: 0x0000008840450020, offset: 57350 },
            Metadata { mask: BitBoard(0x0028002844020000), magic: 0x00007ffc80180030, offset: 22813 },
            Metadata { mask: BitBoard(0x0050005008040200), magic: 0x007fffdd80140028, offset: 56693 },
            Metadata { mask: BitBoard(0x0020002010080400), magic: 0x00020080200a0004, offset:  5818 },
            Metadata { mask: BitBoard(0x0040004020100800), magic: 0x0000101010100020, offset:  7098 },
            Metadata { mask: BitBoard(0x0000020408102000), magic: 0x0007ffdfc1805000, offset:  4451 },
            Metadata { mask: BitBoard(0x0000040810204000), magic: 0x0003ffefe0c02200, offset:  4709 },
            Metadata { mask: BitBoard(0x00000a1020400000), magic: 0x0000000820806000, offset:  4794 },
            Metadata { mask: BitBoard(0x0000142240000000), magic: 0x0000000008403000, offset: 13364 },
            Metadata { mask: BitBoard(0x0000284402000000), magic: 0x0000000100202000, offset:  4570 },
            Metadata { mask: BitBoard(0x0000500804020000), magic: 0x0000004040802000, offset:  4282 },
            Metadata { mask: BitBoard(0x0000201008040200), magic: 0x0004010040100400, offset: 14964 },
            Metadata { mask: BitBoard(0x0000402010080400), magic: 0x00006020601803f4, offset:  4026 },
            Metadata { mask: BitBoard(0x0002040810204000), magic: 0x0003ffdfdfc28048, offset:  4826 },
            Metadata { mask: BitBoard(0x0004081020400000), magic: 0x0000000820820020, offset:  7354 },
            Metadata { mask: BitBoard(0x000a102040000000), magic: 0x0000000008208060, offset:  4848 },
            Metadata { mask: BitBoard(0x0014224000000000), magic: 0x0000000000808020, offset: 15946 },
            Metadata { mask: BitBoard(0x0028440200000000), magic: 0x0000000001002020, offset: 14932 },
            Metadata { mask: BitBoard(0x0050080402000000), magic: 0x0000000401002008, offset: 16588 },
            Metadata { mask: BitBoard(0x0020100804020000), magic: 0x0000004040404040, offset:  6905 },
            Metadata { mask: BitBoard(0x0040201008040200), magic: 0x007fff9fdf7ff813, offset: 16076 },
        ];

        let mut table = [BitBoard::EMPTY; 88772];

        fn generate_table(
            metadata: [Metadata; 64],
            table: &mut [BitBoard],
            shift: u32,
            slide_fn: impl Fn(BitBoard, BitBoard) -> BitBoard,
        ) {
            for square in Square::ALL {
                let Metadata {
                    mask,
                    magic,
                    offset,
                } = metadata[square];

                for subset in mask.subsets() {
                    table[offset + (subset.0.wrapping_mul(magic) >> (64 - shift)) as usize] =
                        slide_fn(square.as_bitboard(), subset);
                }
            }
        }

        generate_table(cross_metadata, &mut table, 12, gen_cross_slides);
        generate_table(diagonal_metadata, &mut table, 9, gen_diagonal_slides);

        write_table("SLIDES", &table, "BitBoard", &mut output_file)?;
        write_table("CROSS_META", &cross_metadata, "Metadata", &mut output_file)?;
        write_table("DIAGONAL_META", &diagonal_metadata, "Metadata", &mut output_file)?;
    }

    write_table(
        "KNIGHT_ATTACKS",
        &gen_piece_table(gen_knight_index),
        "BitBoard",
        &mut output_file,
    )?;
    write_table(
        "KING_ATTACKS",
        &gen_piece_table(gen_king_index),
        "BitBoard",
        &mut output_file,
    )?;

    write_table(
        "WHITE_PAWN_ATTACKS",
        &array::from_fn::<_, 64, _>(|index| {
            let square: BitBoard = Square::try_from(index as u8).unwrap().into();

            square.move_one_up_left(Color::White) + square.move_one_up_right(Color::White)
        }),
        "BitBoard",
        &mut output_file,
    )?;

    write_table(
        "BLACK_PAWN_ATTACKS",
        &array::from_fn::<_, 64, _>(|index| {
            let square: BitBoard = Square::try_from(index as u8).unwrap().into();

            square.move_one_up_left(Color::Black) + square.move_one_up_right(Color::Black)
        }),
        "BitBoard",
        &mut output_file,
    )?;

    write_table(
        "WHITE_PAWN_PUSHES",
        &array::from_fn::<_, 64, _>(|index| {
            let square = Square::try_from(index as u8).unwrap();
            let square_as_bitboard: BitBoard = square.into();

            square_as_bitboard.move_one_up(Color::White)
                + if square.rank() == 1 {
                    square_as_bitboard.move_two_up(Color::White)
                } else {
                    BitBoard::EMPTY
                }
        }),
        "BitBoard",
        &mut output_file,
    )?;

    write_table(
        "BLACK_PAWN_PUSHES",
        &array::from_fn::<_, 64, _>(|index| {
            let square = Square::try_from(index as u8).unwrap();
            let square_as_bitboard: BitBoard = square.into();

            square_as_bitboard.move_one_up(Color::Black)
                + if square.rank() == 6 {
                    square_as_bitboard.move_two_up(Color::Black)
                } else {
                    BitBoard::EMPTY
                }
        }),
        "BitBoard",
        &mut output_file,
    )?;

    write_table(
        "LINE",
        &array::from_fn::<_, { 64 * 64 }, _>(|index| {
            let first_square = Square::try_from((index / 64) as u8).unwrap();
            let second_square = Square::try_from((index % 64) as u8).unwrap();

            Square::ALL
                .into_iter()
                .filter(|&square| {
                    square.on_line_with(first_square) && square.on_line_with(second_square)
                })
                .map(BitBoard::from)
                .fold(BitBoard::EMPTY, |board, square| board + square)
        }),
        "BitBoard",
        &mut output_file,
    )?;

    write_table(
        "BETWEEN",
        &array::from_fn::<_, { 64 * 64 }, _>(|index| {
            let first_square = Square::try_from((index / 64) as u8).unwrap();
            let second_square = Square::try_from((index % 64) as u8).unwrap();

            Square::ALL
                .into_iter()
                .filter(|&square| {
                    square.on_line_with(first_square)
                        && square.on_line_with(second_square)
                        && square.in_rectangle(first_square, second_square)
                })
                .fold(BitBoard::EMPTY, |board, square| board + square.into())
        }),
        "BitBoard",
        &mut output_file,
    )?;

    write_variable(
        "ZOBRIST_MAP",
        StdRng::seed_from_u64(SEED).gen::<ZobristMap>(),
        "ZobristMap",
        &mut output_file,
    )?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

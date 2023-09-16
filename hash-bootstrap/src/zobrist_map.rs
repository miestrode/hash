use rand::{distributions::Standard, prelude::Distribution, Rng};

fn zobrist_array<R: Rng + ?Sized>(rng: &mut R) -> [u64; 64] {
    let mut zobrist_array = [0u64; 64];
    rng.fill(&mut zobrist_array);

    zobrist_array
}

#[derive(Debug)]
pub struct ZobristSide {
    pub white_to_move: u64,
    pub black_to_move: u64,
}

impl Distribution<ZobristSide> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ZobristSide {
        ZobristSide {
            white_to_move: rng.gen(),
            black_to_move: rng.gen(),
        }
    }
}

#[derive(Debug)]
// This contains all configurations of castling rights for each player, as 2^4 = 16
pub struct ZobristCastlingRights(pub [u64; 16]);

impl Distribution<ZobristCastlingRights> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ZobristCastlingRights {
        ZobristCastlingRights(rng.gen())
    }
}

#[derive(Debug)]
pub struct ZobristPieces {
    pub king: [u64; 64],
    pub queen: [u64; 64],
    pub rook: [u64; 64],
    pub bishop: [u64; 64],
    pub knight: [u64; 64],
    pub pawn: [u64; 64],
}

impl Distribution<ZobristPieces> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ZobristPieces {
        ZobristPieces {
            king: zobrist_array(rng),
            queen: zobrist_array(rng),
            rook: zobrist_array(rng),
            bishop: zobrist_array(rng),
            knight: zobrist_array(rng),
            pawn: zobrist_array(rng),
        }
    }
}

#[derive(Debug)]
pub struct ZobristMap {
    pub pieces: ZobristPieces,
    pub castling_rights: ZobristCastlingRights,
    pub side: ZobristSide,
    pub ep_file: [u64; 8],
}

impl Distribution<ZobristMap> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ZobristMap {
        ZobristMap {
            pieces: rng.gen(),
            castling_rights: rng.gen(),
            side: rng.gen(),
            ep_file: rng.gen(),
        }
    }
}

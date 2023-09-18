use rustifact::ToTokenStream;
use standard_dist::StandardDist;

#[derive(Debug, StandardDist, ToTokenStream)]
pub struct ZobristSide {
    pub white_to_move: u64,
    pub black_to_move: u64,
}

#[derive(Debug, StandardDist, ToTokenStream)]
// This contains all configurations of castling rights for each player, as 2^4 = 16
pub struct ZobristCastlingRights(pub [u64; 16]);

#[derive(Debug, StandardDist, ToTokenStream)]
pub struct ZobristPieces {
    pub king: [u64; 64],
    pub queen: [u64; 64],
    pub rook: [u64; 64],
    pub bishop: [u64; 64],
    pub knight: [u64; 64],
    pub pawn: [u64; 64],
}

#[derive(Debug, StandardDist, ToTokenStream)]
pub struct ZobristMap {
    pub pieces: ZobristPieces,
    pub castling_rights: ZobristCastlingRights,
    pub side: ZobristSide,
    pub ep_file: [u64; 8],
}
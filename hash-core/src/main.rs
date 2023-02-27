use std::{hint::black_box, str::FromStr};

use hash_core::game::Game;


fn main() {
    black_box(Game::from_str(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    ))
    .unwrap()
    .perft(5);
}

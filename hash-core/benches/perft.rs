use std::str::FromStr;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hash_core::game::Game;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("perft default 3", |b| {
        b.iter(|| black_box(Game::default()).board.perft(3))
    });
    c.bench_function("perft default 5", |b| {
        b.iter(|| black_box(Game::default()).board.perft(5))
    });
    c.bench_function("perft kiwipete 3", |b| {
        b.iter(|| {
            black_box(
                Game::from_str(
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                )
                .unwrap(),
            ).board
            .perft(3)
        })
    });
    c.bench_function("perft kiwipete 4", |b| {
        b.iter(|| {
            black_box(
                Game::from_str(
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                )
                .unwrap(),
            ).board
            .perft(4)
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

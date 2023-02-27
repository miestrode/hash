use std::str::FromStr;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hash_core::game::Game;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fen default", |b| b.iter(|| black_box(Game::default())));
    c.bench_function("fen error", |b| {
        b.iter(|| {
            black_box(Game::from_str(
                "rnbqkbnr/pppppppp/G/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            ))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

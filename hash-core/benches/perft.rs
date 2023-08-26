use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hash_core::{game::Game, mg};

pub fn perft(game: &mut Game, depth: u32) -> u64 {
    let moves = mg::gen_moves(&game.board);

    match depth {
        // At a depth of one we know all next moves will reach depth zero.
        // Thus, we can know they are all leaves and add one each to the nodes searched.
        1 => moves.len() as u64,
        _ => moves
            .into_iter()
            .map(|chess_move| {
                unsafe { game.make_move_unchecked(&chess_move) };
                let result = perft(game, depth - 1);
                game.unmake_last_move();

                result
            })
            .sum(),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("perft default 3", |b| {
        b.iter(|| black_box(perft(&mut Game::default(), 3)))
    });
    c.bench_function("perft default 5", |b| {
        b.iter(|| black_box(perft(&mut Game::default(), 5)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

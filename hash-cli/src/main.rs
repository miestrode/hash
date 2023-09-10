use std::{io, str::FromStr, thread};

use hash_core::game::Game;
use hash_eval::BasicEvaluator;

fn main() {
    let mut game =
        Game::from_str("rn1qkbnr/4pp1p/b1pp2p1/pp6/P2PP3/1BN2N2/1PPB1PPP/R2Q1RK1 w kq - 1 10")
            .unwrap();

    loop {
        let mut move_string = String::with_capacity(5);
        io::stdin().read_line(&mut move_string).unwrap();
        let chess_move = game.board.interpret_move(move_string.trim()).unwrap();

        unsafe {
            game.make_move_unchecked(&chess_move);
        };

        let chess_move = thread::scope(|s| {
            thread::Builder::new()
                .stack_size(2 * 1024 * 1024 * 1024)
                .spawn_scoped(s, || {
                    hash_search::search(&mut game, &BasicEvaluator, 8, 0.75).unwrap()
                })
                .unwrap()
                .join()
                .unwrap()
        });

        println!("I choose {}. What about your move?", chess_move);

        unsafe {
            game.make_move_unchecked(&chess_move);
        };
    }
}

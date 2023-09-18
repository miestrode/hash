use hash_bootstrap::{BitBoard, Color, Square};
use hash_core::game::Game;
use hash_core::index;
use hash_core::mg;

fn main() {
    let game = Game::default();

    println!("{}", index::pawn_moves(Square::E2, BitBoard::EMPTY, BitBoard::EMPTY, Color::White));

    for chess_move in mg::gen_moves(&game.board) {
        println!("{chess_move}");
    }
}
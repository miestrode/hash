use hash_core::board::Board;
use hash_search::search;

fn main() {
    let board = Board::starting_position();

    println!("{}", search::search(board));
}

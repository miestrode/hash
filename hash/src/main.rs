use std::{error::Error, io, str::FromStr};

use hash::repr::Board;

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        let mut position = String::new();
        io::stdin().read_line(&mut position)?;

        let mut depth = String::new();
        io::stdin().read_line(&mut depth)?;
        let depth = depth.trim().parse()?;

        let board = Board::from_str(position.trim())?;

        println!(
            "Total positions encountered: {}",
            board
                .split_perft(depth)
                .into_iter()
                .map(|(chess_move, positions)| {
                    println!("{chess_move}: {positions}");
                    positions
                })
                .sum::<u64>()
        );
    }
}

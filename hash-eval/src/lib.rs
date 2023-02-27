use hash_core::{board::Board, repr::Player, Color};
use hash_search::Eval;

fn material(player: Player) -> u32 {
    player.queens.count_ones() * 9
        + player.rooks.count_ones() * 5
        + player.bishops.count_ones() * 3
        + player.knights.count_ones() * 3
        + player.pawns.count_ones()
}

struct Evaluator;

impl Eval for Evaluator {
    fn eval(&self, board: Board) -> f32 {
        (match board.current_color {
            Color::White => 1.0,
            Color::Black => -1.0,
        }) * (material(board.current_player) - material(board.opposing_player)) as f32
    }
}

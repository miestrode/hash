use hash_core::{board::Board, repr::Player};
use hash_search::{score::Score, Eval};

// An i32 is used here since we are, during evaluation, subtracting material
fn material(player: Player) -> i16 {
    (player.queens.count_ones() * 9
        + player.rooks.count_ones() * 5
        + player.bishops.count_ones() * 3
        + player.knights.count_ones() * 3
        + player.pawns.count_ones()) as i16
}

pub struct BasicEvaluator;

impl Eval for BasicEvaluator {
    fn eval(&self, board: &Board) -> Score {
        Score::from_evaluation(material(board.current_player) - material(board.opposing_player))
    }
}

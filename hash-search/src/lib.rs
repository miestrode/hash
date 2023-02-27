use hash_core::board::Board;

pub trait Eval {
    fn eval(&self, board: Board) -> f32;
}

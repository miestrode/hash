use crate::{MoveProbabilities, Network, NetworkResult};
use burn::tensor::backend::Backend;
use hash_core::board::Board;
use hash_network::model::Model;
use num_traits::ToPrimitive;
use std::iter;

impl<B: Backend> Network for Model<B> {
    fn maximum_boards_expected(&self) -> usize {
        self.move_history()
    }

    fn run(&self, boards: Vec<Board>) -> NetworkResult {
        let empty_boards = self.maximum_boards_expected() - boards.len();

        let boards = boards
            .iter()
            .map(Some)
            .chain(iter::repeat(None).take(empty_boards))
            .collect();

        let output = self
            .forward(hash_network::boards_to_tensor(boards).unsqueeze())
            .squeeze::<1>(0)
            .into_data()
            .value;

        NetworkResult {
            value: output[0].to_f32().unwrap(),
            move_probabilities: MoveProbabilities {
                probabilities: output[1..]
                    .iter()
                    .map(|float| float.to_f32().unwrap())
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            },
        }
    }
}

use burn::{
    config::Config,
    module::Module,
    nn::{
        conv::{Conv2d, Conv2dConfig},
        BatchNorm, BatchNormConfig, Linear, LinearConfig, PaddingConfig2d, ReLU,
    },
    tensor::{activation, backend::Backend, Shape, Tensor},
};
use std::iter;

// TODO: Consider placing some of the information here in the input instead of in each historical board state.
// The 3rd dimension value of the shape of a board tensor.
#[rustfmt::skip]
pub const SINGLE_BOARD_DIMENSION: usize =
    6 // 6 piece kinds for white 
        + 6 // 6 piece kinds for black 
        + 1 // 1 layer for the en-passant square
        + 2 // 2 ways to castle (king-side, queen-side) for white
        + 2 // 2 ways to castle (king-side, queen-side) for black
        + 1 // 1 layer to denote who is playing. 1 = white, -1 = black.
        + 1 // 1 layer for the half-move clock, as per the 50 move rule.
        + 1; // 1 layer to denote if this board is even existent. Naturally a history of N-moves
             // cannot be created when a game hasn't gone through N positions.

// The output size is simply the length of the vector output by the model. It encodes all Chess
// moves and a position value node. Note that it does overshoot the number of possible Chess moves
// by quite a bit and considers some illegal moves.
#[rustfmt::skip]
const OUTPUT_SIZE: usize =
    1 // One output is simply the value of the state
        + 64 * 64 // These are all regular moves, from a square to another square, with no promotions naturally
        + 8 * 8 * 4  // All of the possible promotions for the eighth rank
        + 8 * 8 * 4; // All of the possible promotions for the first rank

fn calculate_board_tensor_dimension(move_history: usize) -> usize {
    SINGLE_BOARD_DIMENSION * move_history
}

#[derive(Module, Debug)]
struct ConvBlock<B: Backend> {
    conv: Conv2d<B>,
    batch_norm: BatchNorm<B, 2>,
    activation: ReLU,
}

impl<B: Backend> ConvBlock<B> {
    fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.conv.forward(input);
        let x = self.batch_norm.forward(x);

        self.activation.forward(x)
    }
}

#[derive(Config, Debug)]
enum ConvBlockKind {
    Head { move_history: usize },
    Body { filters: usize },
}

#[derive(Config, Debug)]
struct ConvBlockConfig {
    conv_block_kind: ConvBlockKind,
    kernel_length: usize,
    filters: usize,
}

impl ConvBlockConfig {
    fn init<B: Backend>(&self) -> ConvBlock<B> {
        let input_size = match self.conv_block_kind {
            ConvBlockKind::Head { move_history } => calculate_board_tensor_dimension(move_history),
            ConvBlockKind::Body { filters } => filters,
        };

        ConvBlock {
            conv: Conv2dConfig::new(
                [input_size, self.filters],
                [self.kernel_length, self.kernel_length],
            )
            .with_padding(PaddingConfig2d::Same)
            .init(),
            batch_norm: BatchNormConfig::new(self.filters).init(),
            activation: ReLU::default(),
        }
    }
}

pub struct BatchOutput<B: Backend> {
    pub values: Tensor<B, 1>,
    pub probabilities: Tensor<B, 2>,
}

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    move_history: usize,
    conv_blocks: Vec<ConvBlock<B>>,
    fc_1: Linear<B>,
    output: Linear<B>,
}

impl<B: Backend> Model<B> {
    pub fn move_history(&self) -> usize {
        self.move_history
    }

    pub fn forward(&self, input: Tensor<B, 4>) -> BatchOutput<B> {
        let x = self
            .conv_blocks
            .iter()
            .fold(input, |x, block| block.forward(x));
        let x = x.flatten(1, 3);
        let x = self.fc_1.forward(x);
        let x = self.output.forward(x);

        let shape = x.shape().dims;
        let value_index_tensor = Tensor::zeros(Shape::new([shape[0], 1]));

        let values = x.clone().gather(1, value_index_tensor.clone()).squeeze(1);
        let probabilities = activation::softmax(x.slice([0..shape[0], 1..shape[1]]), 1);

        BatchOutput {
            values,
            probabilities,
        }
    }
}

#[derive(Config, Debug)]
pub struct ModelConfig {
    #[config(default = 10000)]
    hidden_layer_size: usize,
    #[config(default = 15)]
    conv_blocks: usize,
    #[config(default = 7)]
    move_history: usize,
    #[config(default = 3)]
    kernel_length: usize,
    #[config(default = 32)]
    filters: usize,
}

impl ModelConfig {
    pub fn init<B: Backend>(&self) -> Model<B> {
        Model {
            move_history: self.move_history,
            conv_blocks: iter::once(
                ConvBlockConfig::new(
                    ConvBlockKind::Head {
                        move_history: self.move_history,
                    },
                    self.kernel_length,
                    self.filters,
                )
                .init(),
            )
            .chain(
                iter::repeat(
                    ConvBlockConfig::new(
                        ConvBlockKind::Body {
                            filters: self.filters,
                        },
                        self.kernel_length,
                        self.filters,
                    )
                    .init(),
                )
                .take(self.conv_blocks - 1),
            )
            .collect(),
            fc_1: LinearConfig::new(self.filters * 8 * 8, self.hidden_layer_size).init(),
            output: LinearConfig::new(self.hidden_layer_size, OUTPUT_SIZE).init(),
        }
    }
}

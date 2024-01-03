use burn::{
    config::Config,
    module::Module,
    nn::{
        conv::{Conv2d, Conv2dConfig},
        pool::{AvgPool2d, AvgPool2dConfig},
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
        + 1 // 1 layer for the en passant square
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
struct PreConvBlock<B: Backend> {
    batch_norm: BatchNorm<B, 2>,
    activation: ReLU,
    conv: Conv2d<B>,
}

impl<B: Backend> PreConvBlock<B> {
    fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.batch_norm.forward(input);
        let x = self.activation.forward(x);

        self.conv.forward(x)
    }
}

#[derive(Config, Debug)]
struct PreConvBlockConfig {
    kernel_length: usize,
    filters: usize,
}

impl PreConvBlockConfig {
    fn init<B: Backend>(&self) -> PreConvBlock<B> {
        PreConvBlock {
            conv: Conv2dConfig::new(
                [self.filters, self.filters],
                [self.kernel_length, self.kernel_length],
            )
            .with_padding(PaddingConfig2d::Same)
            .init(),
            batch_norm: BatchNormConfig::new(self.filters).init(),
            activation: ReLU::default(),
        }
    }
}

#[derive(Module, Debug)]
struct SeBlock<B: Backend> {
    preconv_1: PreConvBlock<B>,
    preconv_2: PreConvBlock<B>,
    avg_pool: AvgPool2d,
    fc_1: Linear<B>,
    activation: ReLU,
    fc_2: Linear<B>,
}

impl<B: Backend> SeBlock<B> {
    fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        let residual = self.preconv_1.forward(input.clone());
        let residual = self.preconv_2.forward(residual);

        let scale = self.avg_pool.forward(residual.clone());
        let final_shape = scale.shape();
        let scale = scale.flatten::<2>(1, 3);
        let scale = self.fc_1.forward(scale);
        let scale = self.activation.forward(scale);
        let scale = self.fc_2.forward(scale);
        let scale = activation::sigmoid(scale);
        let scale = scale.reshape(final_shape);

        let scaled_residual = residual.mul(scale);

        let result = input + scaled_residual;

        self.activation.forward(result)
    }
}

#[derive(Config, Debug)]
struct SeBlockConfig {
    kernel_length: usize,
    filters: usize,
    ratio: usize,
}

impl SeBlockConfig {
    fn init<B: Backend>(&self) -> SeBlock<B> {
        let fc_2_input_size = self.filters / self.ratio;
        let preconv = PreConvBlockConfig::new(self.kernel_length, self.filters).init();

        SeBlock {
            activation: ReLU::default(),
            preconv_1: preconv.clone(),
            preconv_2: preconv,
            avg_pool: AvgPool2dConfig::new([8, 8]).init(),
            fc_1: LinearConfig::new(self.filters, fc_2_input_size).init(),
            fc_2: LinearConfig::new(fc_2_input_size, self.filters).init(),
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
    conv_block: Conv2d<B>,
    se_blocks: Vec<SeBlock<B>>,
    fc_1: Linear<B>,
    output: Linear<B>,
}

impl<B: Backend> Model<B> {
    pub fn move_history(&self) -> usize {
        self.move_history
    }

    pub fn forward(&self, input: Tensor<B, 4>) -> BatchOutput<B> {
        let x = self.conv_block.forward(input);
        let x = self.se_blocks.iter().fold(x, |x, block| block.forward(x));
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
    #[config(default = 1)]
    initial_kernel_stride: usize,
    #[config(default = 3)]
    initial_kernel_length: usize,
    #[config(default = 1000)]
    hidden_layer_size: usize,
    #[config(default = 15)]
    se_blocks: usize,
    #[config(default = 8)]
    move_history: usize,
    #[config(default = 3)]
    kernel_length: usize,
    #[config(default = 256)]
    filters: usize,
    #[config(default = 16)]
    ratio: usize,
}

impl ModelConfig {
    pub fn init<B: Backend>(&self) -> Model<B> {
        Model {
            move_history: self.move_history,
            conv_block: Conv2dConfig::new(
                [
                    calculate_board_tensor_dimension(self.move_history),
                    self.filters,
                ],
                [self.initial_kernel_length, self.initial_kernel_length],
            )
            .with_stride([self.initial_kernel_stride, self.initial_kernel_stride])
            .with_padding(PaddingConfig2d::Same)
            .init(),
            se_blocks: iter::repeat(
                SeBlockConfig::new(self.kernel_length, self.filters, self.ratio).init(),
            )
            .take(self.se_blocks)
            .collect(),
            fc_1: LinearConfig::new(self.filters * 8 * 8, self.hidden_layer_size).init(),
            output: LinearConfig::new(self.hidden_layer_size, OUTPUT_SIZE).init(),
        }
    }
}

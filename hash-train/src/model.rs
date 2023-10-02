use burn::{
    config::Config,
    module::Module,
    nn::{
        conv::{Conv2d, Conv2dConfig},
        BatchNorm, BatchNormConfig, Linear, LinearConfig, ReLU,
    },
    tensor::{backend::Backend, Tensor},
};

// TODO: Consider placing some of the information here in the input instead of in each historical board state.
// The 3rd dimension value of the shape of a board tensor.
#[rustfmt::skip]
const SINGLE_BOARD_DIMENSION: usize =
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
struct ConvBlockConfig {
    move_history: usize,
    kernel_length: usize,
    filters: usize,
}

impl ConvBlockConfig {
    fn init<B: Backend>(&self) -> ConvBlock<B> {
        let input_size = calculate_board_tensor_dimension(self.move_history);

        ConvBlock {
            conv: Conv2dConfig::new(
                [input_size, self.filters],
                [self.kernel_length, self.kernel_length],
            )
            .init(),
            batch_norm: BatchNormConfig::new(self.filters).init(),
            activation: ReLU::default(),
        }
    }
}

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    conv_blocks: Vec<ConvBlock<B>>,
    fc_1: Linear<B>,
    output: Linear<B>,
}

impl<B: Backend> Model<B> {
    pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 2> {
        let x = self
            .conv_blocks
            .iter()
            .fold(input, |x, block| block.forward(x));
        let x = x.flatten(0, 3);
        let x = self.fc_1.forward(x);

        self.output.forward(x)
    }
}

#[derive(Config, Debug)]
pub struct ModelConfig {
    #[config(default = 10000)]
    hidden_layer_size: usize,
    #[config(default = 50)]
    conv_blocks: usize,
    #[config(default = 7)]
    move_history: usize,
    #[config(default = 3)]
    kernel_length: usize,
    #[config(default = 256)]
    filters: usize,
}

impl ModelConfig {
    pub fn init<B: Backend>(&self) -> Model<B> {
        let input_size = calculate_board_tensor_dimension(self.move_history);

        Model {
            conv_blocks: vec![
                ConvBlockConfig::new(
                    self.move_history,
                    self.kernel_length,
                    self.filters,
                )
                .init();
                self.conv_blocks
            ],
            fc_1: LinearConfig::new(input_size, self.hidden_layer_size).init(),
            output: LinearConfig::new(self.hidden_layer_size, OUTPUT_SIZE).init(),
        }
    }
}

use burn::{
    grad_clipping::GradientClippingConfig,
    nn::loss::MSELoss,
    optim::{
        decay::WeightDecayConfig, momentum::MomentumConfig, GradientsParams, Optimizer, SgdConfig,
    },
    tensor::{
        backend::{AutodiffBackend, Backend},
        Shape, Tensor,
    },
};
use hash_network::model::{BatchOutput, Model, ModelConfig};
use rand::Rng;
use ringbuffer::RingBuffer;

use crate::{play, TrainBuffer};

pub fn add_games<B: Backend>(
    train_buffer: &mut TrainBuffer<B>,
    model: &Model<B>,
    rng: &mut impl Rng,
    ply_cap: usize,
    games: usize,
) {
    for game in 0..games {
        println!("GENERATING GAME {game}");

        let game_data = play::gen_game(model, ply_cap, rng);

        train_buffer.extend(game_data);
    }
}

fn decouple_output<B: Backend>(outputs: Tensor<B, 2>) -> (Tensor<B, 1>, Tensor<B, 2>) {
    let shape = outputs.dims();
    let value_index_tensor = Tensor::zeros(Shape::new([shape[0], 1]));

    let values = outputs
        .clone()
        .gather(1, value_index_tensor.clone())
        .squeeze(1);

    (values, outputs.slice([0..shape[0], 1..shape[1]]))
}

fn loss<B: Backend>(
    values: Tensor<B, 1>,
    expected_values: Tensor<B, 1>,
    probabilities: Tensor<B, 2>,
    expected_probabilities: Tensor<B, 2>,
) -> Tensor<B, 1> {
    let value_length = values.dims()[0];

    let value_loss = MSELoss::new()
        .forward_no_reduction(
            values.reshape(Shape::new([value_length, 1])),
            expected_values.reshape(Shape::new([value_length, 1])),
        )
        .reshape(Shape::new([value_length]));

    let probability_loss = probabilities
        .log()
        .mul(expected_probabilities)
        .sum_dim(1)
        .squeeze(1);

    let loss_per_item = value_loss.sub(probability_loss);

    loss_per_item.mean()
}

pub fn run<B: AutodiffBackend>() {
    let epochs = 1000;
    let ply_cap = 80;
    let mut games_per_iteration = 8;
    let batches_per_iteration = 1000;
    let batch_length = 2048;
    let learning_rate = 0.02; // TODO: Use annealing or cyclical learning rates

    let mut rng = rand::thread_rng();
    let mut optimizer = SgdConfig::new()
        .with_momentum(Some(MomentumConfig::new().with_momentum(0.3)))
        .with_weight_decay(Some(WeightDecayConfig::new(1e-5)))
        .with_gradient_clipping(Some(GradientClippingConfig::Norm(10.0)))
        .init();
    let mut model = ModelConfig::new().init::<B>();
    let mut train_buffer = TrainBuffer::new();

    for epoch in 1..epochs + 1 {
        println!("GENERATING {games_per_iteration} GAMES FOR EPOCH {epoch}");

        // Generate self-play games
        add_games(
            &mut train_buffer,
            &model,
            &mut rng,
            ply_cap,
            games_per_iteration,
        );

        println!("========= BEGIN EPOCH {epoch} TRAINING =========");

        for iteration in 0..batches_per_iteration {
            let (batch, expected_outputs): (Vec<_>, Vec<_>) = rand::seq::index::sample(
                &mut rng,
                train_buffer.len(),
                batch_length.min(train_buffer.len()),
            )
            .into_iter()
            .map(|index| {
                let train_input = train_buffer[index].clone();

                (train_input.input, train_input.expected_output)
            })
            .unzip();

            let batch = Tensor::stack(batch, 0);

            let (expected_values, expected_probabilities) =
                decouple_output(Tensor::stack(expected_outputs, 0));
            let BatchOutput {
                values,
                probabilities,
            } = model.forward(batch);

            // TODO: Use the same loss from the original AlphaZero paper, with CE loss for the
            // probabilities and MSE loss for the value
            let loss = loss(
                values,
                expected_values,
                probabilities,
                expected_probabilities,
            );

            println!(
                "[Epoch {epoch} - Iteration {iteration}] Loss {}",
                loss.clone().into_scalar()
            );

            let gradients = GradientsParams::from_grads(loss.backward(), &model);

            model = optimizer.step(learning_rate, model, gradients);
        }

        if games_per_iteration < 20000 {
            games_per_iteration <<= 1;
        }
    }
}

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
use std::error::Error;

use dfdx::{
    dtypes::{f16, AMP},
    optim::Sgd,
    prelude::*,
};
use hash_eval::{
    model::Model,
    train::{self, Hyperparams},
};

fn main() -> Result<(), Box<dyn Error>> {
    let device = AutoDevice::default();

    let mut network = device.build_module::<Model, AMP<f16>>();
    let mut optimizer = Sgd::new(&network, SgdConfig::default());

    train::train(
        &mut network,
        Hyperparams {
            discount_factor: AMP(f16::from_f32(0.999)),
            max_fitting_iterations: 100,
            acceptable_loss: AMP(f16::from_f32(0.15)),
            batch_size: 50,
            max_games: 100000,
        },
        &mut optimizer,
        &device,
    )
}

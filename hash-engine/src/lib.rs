#![feature(error_iter)]
mod engine;
use std::{error::Error, io};

use engine::{Engine, MessageReader};

pub fn run() -> Result<(), Box<dyn Error>> {
    Engine::new(MessageReader::new(io::stdin().lock()))?.run()
}

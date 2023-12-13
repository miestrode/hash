use std::io;

use hash_engine::Engine;

fn main() {
    Engine::new(io::stdin().lock()).run();
}

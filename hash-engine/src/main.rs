use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    hash_engine::run()
}

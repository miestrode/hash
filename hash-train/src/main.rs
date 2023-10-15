use burn::{autodiff::ADBackendDecorator, backend::TchBackend};

fn main() {
    hash_train::train::run::<ADBackendDecorator<TchBackend<f32>>>();
}

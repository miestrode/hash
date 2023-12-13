use burn::backend::Autodiff;
use burn_wgpu::Wgpu;

fn main() {
    hash_train::train::run::<Autodiff<Wgpu>>();
}

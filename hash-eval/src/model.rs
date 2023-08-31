use dfdx::prelude::*;

type ConvPart<const I: usize, const O: usize, const S: usize> = (Conv2D<I, O, S>, ReLU);
pub type Model = (
    ConvPart<13, 30, 2>,
    ConvPart<30, 30, 4>,
    Flatten2D,
    Linear<480, 1000>,
    Linear<1000, 1>,
);
pub(crate) type Network<E, D> = <Model as BuildOnDevice<D, E>>::Built;

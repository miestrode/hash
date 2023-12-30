use play::TrainInput;
use ringbuffer::ConstGenericRingBuffer;

mod play;
pub mod train;

pub const TRAIN_BUFFER_CAPACITY: usize = 1 << 6;

pub type TrainBuffer<B> = ConstGenericRingBuffer<TrainInput<B>, TRAIN_BUFFER_CAPACITY>;

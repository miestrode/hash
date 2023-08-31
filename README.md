# The Hash Chess Engine
[![Status](https://github.com/miestrode/hash/workflows/Rust/badge.svg)](https://github.com/miestrode/hash/actions)

Hash is an experimental Chess engine written in Rust. It uses a heavily optimized, cross-platform move generator, a convolutional neural network evaluation function and a search algorithm based on BNS search, extended with parallelism.

Hash is also a learning project for me, and as a goal attempts to use as much Rust as possible in the codebase, even though for certain things, such as training the evaluation network, this will likely not be done, due to the nascent state of the Rust ML ecosystem.

## To do
The primary things as of right now to be done, are:

- [ ] Get a final design for parallel BNS search and implement it
- [ ] Bridge between the ML framework chosen and `hash-core`, to implement the training code
- [ ] Write the training code for the CNN
- [ ] Refactor the codebase and make a nicer internal and external API for `hash-core`
- [ ] Create a new test suite
- [ ] Evaluate the Engine's Elo rating

# The Hash Chess Engine
[![Status](https://github.com/miestrode/hash/workflows/Rust/badge.svg)](https://github.com/miestrode/hash/actions)

Hash is an experimental Chess engine written in Rust. Hash will use a non-NNUE, neural network evaluation function, likely implemented using CNNs, a unique form of evaluation function training and will explore the tree using BNS search.

The ultimate goal of this project is to be able to do every single operation related to the engine fully in Rust - that meaning also the training of the neural networks, playing, and everything else.

## To do
The primary things as of right now to be done, are:

- [ ] Refactor the codebase and make a nicer API for `hash-core`
- [ ] Create a new test suite

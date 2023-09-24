# The Hash Chess Engine

[![Status](https://github.com/miestrode/hash/workflows/Rust/badge.svg)](https://github.com/miestrode/hash/actions)

Hash is an experimental Chess engine written in Rust, with the goal of putting to use recent advancements in statistics,
computer science and computer Chess.
Unlike most traditional Chess engines, Hash doesn't use the alpha-beta framework, and instead opts to perform directed
tree search in the form of AlphaZero-style MCTS. However, unlike Chess engines such as Leela Chess Zero, Hash
incorporates new ideas in it's search, utilizing root-tree parallelization and move-picking via Murphy Sampling, which
should greatly improve it's play.

A secondary goal of Hash is to use as much Rust as possible in it's design, to test the boundaries of what is possible
to do well currently, using Rust. Some areas may suffer, or just won't use Rust as a result, such as network training.

## To do

The primary things as of right now to be done, are:

- [x] Restructure the project (combine `hash-core`, `hash-search`, `hash-eval` and `hash-cli`)
- [ ] Finish the move generation refactor
- [ ] Refactor the build script, and it's magic bitboards setup (consider using `phf`, and unrelatedly switching to
  black
  magic bitboards)
- [ ] Create a testing suite
- [ ] Start work on getting `lc0`'s networks to run in Rust
- [ ] Create a MCTS searcher using the networks (incorporating parallelism, Murphy Sampling and the like)
- [ ] Create a network trainer in Rust, to replace the `lc0` networks
- [ ] Create an evaluation framework, similar to FishTest or OpenBench
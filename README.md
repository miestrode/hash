# The Hash Chess Engine

[![Status](https://github.com/miestrode/hash/workflows/Rust/badge.svg)](https://github.com/miestrode/hash/actions)

Hash is an experimental Chess engine written in Rust, with the goal of putting to use recent advancements in statistics,
computer science and computer Chess.
Unlike most traditional Chess engines, Hash doesn't use the alpha-beta framework, and instead opts to perform directed
tree search in the form of AlphaZero-style MCTS. However, unlike Chess engines such as Leela Chess Zero, Hash
incorporates new ideas in its search, utilizing root-tree parallelization and move-picking via Murphy Sampling, which
should greatly improve its play.

A secondary goal of Hash is to use as much Rust as possible in its design, to test the boundaries of what is possible
to do well currently, using Rust. Some areas may suffer, or just won't use Rust as a result, such as network training.

## To do

### Move generation (`hash-core`)

- [ ] Make the FEN parser fail when the board it is parsing is illegal, as `Board` should never, whilst only using safe
  functions result in an invalid position.
- [ ] Try to reimplement the `Pins` data structure and other ideas from the old move generation code. It is possible
  that reimplementing the generation of slide constraints could make it a viable, fast option again.
- [ ] Refactor the build script, and it's magic bitboards setup (consider using `phf`, and unrelatedly switching to
  black
  magic bitboards)

### MCTS

- [ ] Create an MCTS searcher using the networks (incorporating parallelism, Murphy Sampling and the like)
- [ ] Consider not tying a board to the tree, saving memory
- [ ] Consider to the contrary tying the relevant move to each child, or at least a move integer.

### Network training

- [ ] Create a network trainer in Rust
- [ ] Create an evaluation framework, similar to FishTest or OpenBench

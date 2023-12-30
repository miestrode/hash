# The Hash Chess Engine

[![Status](https://github.com/miestrode/hash/workflows/Rust/badge.svg)](https://github.com/miestrode/hash/actions)

Hash is a CEGO (revision 1) compliant experimental Chess engine written in Rust, with the goal of putting to use recent advancements in statistics, computer science and computer Chess. Unlike most traditional Chess engines, Hash doesn't use the alpha-beta framework, and instead opts to perform directed tree search in the form of AlphaZero-style MCTS, while in the future, incorporating and facilitating new ideas and innovations in Computer Chess, neural network architecture, and directed tree search.

A secondary goal of Hash is to use as much Rust as possible in its design, to test the boundaries of what is possible to do with modern Rust. Therefore, Hash currently uses the Burn deep learning framework for running its neural networks, instead of more established options, such as Tensorflow or PyTorch. The hope is that, in the future, there will be less of a feature gap between the frameworks, and that optimization tools will grow to support Burn and similar Rust-based projects.

Hash is currently in the process of being written, and has not officially released in any form. It is unlikely the code in the repository here currently works as a full Chess engine.

## CEGO (or, why Hash doesn't support UCI)

Hash doesn't support UCI, and instead uses its bespoke protocol, CEGO (Chess Engine Game Operation). The reasons for this are partially explained in [here](docs/cego/REVISION-1.md). It suffices to say, we felt UCI and similar protocols weren't good choices given the use cases for this engine.

## Documentation

As we feel Hash is a sufficiently large project, documentation explaining things such as its current network structure and things of the like can be seen in [here](docs/). Note that documentation is currently largely incomplete.

## Contributing

The project currently will not accept contributions which significantly alter the source code, and so does not have guidelines for doing so. This is because things are currently far too underdeveloped. In the future, a `CONTRIBUTING.md` file will be made.

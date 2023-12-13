# Topology of the project
Below is an explanation of the different files and folders in this project.

## `hash-bootstrap`
Is a crate containing the basic constructs needed for the build script in `hash-core` to function. The build script is required for generating certain lookup tables for move generation.

## `hash-core`
Is a crate containing code for performing move generation and representing a Chess board.

## `hash-network`
Is a crate containing the model definition and supporting code for the Hash neural network (currently H0). It uses the Burn deep learning framework for this.

## `hash-train`
Is a binary crate that uses `hash-network` in order to train a network using its model, and then save it so it can be used by the engine.

## `hash-search`
Is a crate that implements the primary searching logic for the engine, by providing an advanced searching algorithm based on AlphaZero-style MCTS.

## `hash-engine`
Is a binary crate functioning as the front-end for the Hash Chess engine. It contains logic for managing search using operations provided by `hash-search` and the networks produced by `hash-train`, and implements the CGCF protocol. It is intended to be used as a command-line program and has a CLI.

# Design

Below, is a description of the high-level architecture of this project, and of the purpose of each crate in its workspace.

## Architecture

To be written.

## Topology of the project

### `hash-bootstrap`

Contains the basic constructs needed for the build script in `hash-core` to function. The build script is required for generating certain lookup tables for move generation.

### `hash-core`

Contains code for performing move generation and representing a Chess board.

### `hash-network`

Contains the model definition and supporting code for the Hash neural network (currently H0). It uses the Burn deep learning framework for this.

### `hash-train`

Contains core definitions for training the neural network (currently H0).

### `hash-server`

Is a binary for running an instance of the Hash Training and Evaluation Server (HTES), which acts as a distributed job pool for engine SPRT testing, and for training the engine's neural networks, by using computers running `hash-client`.

### `hash-client`

Is a binary for running a client which can perform jobs for an HTES, upon the receival of a request. This means either training jobs, or 

### `hash-search`

Contains the primary searching logic for the engine, by providing an advanced parallel searching algorithm based on AlphaZero-style MCTS, using `hash-network`.

### `hash-engine`

Provides a CLI to the engine. It contains logic for managing search using operations provided by `hash-search` and the networks produced by `hash-train`, and implements the CEGO protocol.

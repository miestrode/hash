# Design of the search neural network

Hash's search network is based on AlphaZero. It is a tower network, meaning that it's mainly composed of a chain of
residual blocks, with two heads, or outputs, for the two kinds of networks trained - a value head, for the value
network, and a policy head for the policy network.

## What is AlphaZero doing, or, what are these heads?

The basic idea behind AlphaZero is to perform guided tree search, in the search space. This means that instead of
looking at all options, like some Chess engines do, we instead only look at a relevant fraction of it, sort of like
humans. The framework in which we do this is generally called MCTS, and we will keep this name here for the sake of
ease of understanding and the lack of a better term. In order to do this guided search we need two networks, the first
one will give us a probability distribution over all moves of how likely each move is to be the best. The second will
give us an evaluation of the end game state of some game state. For example, for a mate in 2 for the current player,
we'd expect a value close to 1. Of course, why not just use a single network? Why even need the probabilities? Let's
first go over how these networks are actually used during search.

### Guided search using value and policy networks

In order to better understand this form of guided search, we must first understand MCTS.

#### MCTS

MCTS, or Monte-Carlo tree search works in four stages, in order to create a search tree, with each node having a set of
children, being the following game state, and each node also having a score between 0 and 1, of it's "goodness", for the
player in it. Then, in order to pick the best move for the current state, look at the root of the tree, and pick the
move corresponding to the node of highest "goodness". In order to actually calculate all of this, we use the following
four stages:

##### Selection

Selection is the first stage, and it just involves going around the current tree (which starts as a single node, the
root) until you reach a target node, being a node which can have more children. You can pick a node however you like,
and this part is the part that does the guiding, based on the current information in the tree. We will talk about common
ways to do this "picking" soon. After picking a target node, you then add to it a new child.

##### Expansion

In this stage, you begin playing out a game from the child node you just added, until the current game node is
terminal (win/draw/loss). At that point, you should have a chain of nodes from the child node up to a terminal node.
Note that some implementations of MCTS and related algorithms don't add the new nodes into the tree, and instead just
play the game out in memory.

##### Backpropagation (updating)

The final stage of the algorithm, called a bit confusingly "backpropagation", involves using the result of the terminal
node to update the tree. 
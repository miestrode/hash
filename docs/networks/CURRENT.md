# The H0 network architecture
The H0 network architecture is the current architecture used in the Hash Chess engine for search. It is a classic AlphaZero-style network, with a policy head, and a value head. Despite this, there are differences between it and the AlphaZero Chess network, in its body, input format, and output format.

H0 isn't designed to be cutting edge or particularly good, but is rather designed so as to have a concrete net work as quickly as possible for Hash's first release, so that other parts of the engine will function.

## Input
The logical input passed to the network is a representation of the 7 last game states, including the current one, where each game state is a 3D tensor, and the states are passed by concatenation, such that the current game state is last. A game state is encoded in the format below. Each enumerated item is an 8x8 float layer, and layers relating to squares on the board are encoded in Little Endian Rank-File (LERF), such that elements corresponding to highlighted squares on the tensor are given a value of `1.0`, and otherwise `0.0`:

1. White pawns (bitboard)
2. White knights (bitboard)
3. White bishops (bitboard)
4. White rooks (bitboard)
5. White queens (bitboard)
6. White king (bitboard)
7. Black pawns (bitboard)
8. Black knights (bitboard)
9. Black bishops (bitboard)
10. Black rooks (bitboard)
11. Black queens (bitboard)
12. Black king (bitboard)
13. En passant capture square (bitboard)
14. Whether white can castle king-side (boolean)
15. Whether white can castle queen-side (boolean)
16. Whether black can castle king-side (boolean)
17. Whether black can castle queen-side (boolean)
18. Whether it is white's turn (boolean)
19. Half-move clock (numeric)
20. Is game state present (boolean)

Layer 20 is used for cases where there do not exist 7 game states as described above. In this case, the whole tensor should simply be composed of zeros.

## Body
The body of the network is composed of 15 convolutional blocks, and some fully connected layers. Each block consists of a convolution with a kernel of size 3x3, using 32 filters, then a batch normalization layer, and then a ReLU activation layer. After the blocks, the output from is passed to a fully connected layer, of output size 10000, which is passed to another fully connected layer, whose output is the final output of the body.

## Output


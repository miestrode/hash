# The Hash Chess Engine
Is an experimental Chess engine written in Rust, featuring:
[x] Bitboards
[x] PEXT/Magic Bitboards
[x] Zobrist Hashing
[x] Bloom filters
[ ] a Global Transposition Table (Current implementation uses `HashMap`s, as it is unclear if a basic
reimplementation of this hash table would actually increase performance, as it is heavily optimized)
[ ] Parallel DTS-based Alpha-Beta Pruning
[ ] Best Node Search
[ ] CNN-based position evaluation

And will possibly feature:
[ ] Nerual network based move-ordering (as seen in the DeepChess NN)
[ ] Different CNNs for evaluation for different game parts

#![feature(test)]

pub mod board;
mod cache;
pub mod game;
mod index;
pub mod mg;
pub mod repr;

#[cfg(test)]
mod tests {
    extern crate test;

    use std::str::FromStr;

    use crate::board::Board;
    use test::Bencher;
    use test_case::test_case;

    #[bench]
    fn bench_perft_default_1(b: &mut Bencher) {
        let board = Board::starting_position();

        b.iter(|| board.perft(5));
    }

    #[test_case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"; "starting position")]
    #[test_case("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"; "kiwipete")]
    #[test_case("8/7r/4Rn1p/1bP1P3/1P1kq3/2N1r3/p1p5/K3b3 w - - 0 1"; "random 1")]
    #[test_case("1k5n/1Pp5/2pP4/4p1r1/5p2/1K1pBP2/1p1Q4/2N5 w - - 0 1"; "random 2")]
    #[test_case("7b/3rr1P1/3P2pK/8/NN2Q2p/p1PB4/8/1b1k4 w - - 0 1"; "random 3")]
    #[test_case("1Q6/QP6/n3n1p1/P5N1/1pp5/1p2K2p/6NN/1k6 w - - 0 1"; "random 4")]
    #[test_case("rnbqkbnr/ppp1pppp/8/8/1PPpP3/8/P2P1PPP/RNBQKBNR b KQkq c3 0 3"; "en passant test")]
    #[test_case("r1bq1b1r/ppppk1pp/2n2n2/4pp2/2B1PP2/5N2/PPPP2PP/RNBQ1RK1 w - - 6 6"; "no castling test")]
    #[test_case("1nbqkbnr/1ppppppp/r7/p7/7P/7R/PPPPPPP1/RNBQKBN1 w Qk - 2 3"; "partial castling test")]
    fn fen_tests(fen_string: &str) {
        assert_eq!(
            fen_string,
            &Board::from_str(fen_string).unwrap().to_string()
        );
    }

    #[test_case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 3, 8902; "starting position depth 3")]
    #[test_case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 5, 4865609; "starting position depth 5")]
    #[test_case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119060324; "starting position depth 6")]
    #[test_case("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 1, 48; "kiwipete depth 1")]
    #[test_case("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 4, 4085603; "kiwipete depth 4")]
    #[test_case("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 5, 193690690; "kiwipete depth 5")]
    #[test_case("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", 6, 1134888; "en passant move 1 depth 6")]
    #[test_case("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", 6, 1015133; "en passant move 2 depth 6")]
    #[test_case("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", 6, 1440467; "en passant move with check depth 6")]
    #[test_case("5k2/8/8/8/8/8/8/4K2R w K - 0 1", 6, 661072; "king-side castle with check depth 6")]
    #[test_case("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", 6, 803711; "queen-side castle with check depth 6")]
    #[test_case("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 4, 1274206; "castle rights depth 4")]
    #[test_case("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1", 4, 1720476; "prevented castling depth 4")]
    #[test_case("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1", 6, 3821001; "promotion out of check depth 6")]
    #[test_case("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1", 5, 1004658; "discovered check depth 5")]
    #[test_case("4k3/1P6/8/8/8/8/K7/8 w - - 0 1", 6, 217342; "promotion with check depth 6")]
    #[test_case("8/P1k5/K7/8/8/8/8/8 w - - 0 1", 6, 92683; "under-promotion with check depth 6")]
    #[test_case("K1k5/8/P7/8/8/8/8/8 w - - 0 1", 6, 2217; "self stalemate depth 6")]
    #[test_case("8/k1P5/8/1K6/8/8/8/8 w - - 0 1", 7, 567584; "stalemate and checkmate 1 depth 7")]
    #[test_case("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1", 4, 23527; "stalemate and checkmate 2 depth 4")]
    #[test_case("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2", 1, 8; "misc 1 depth 1")]
    #[test_case("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3", 1, 8; "misc 2 depth 1")]
    #[test_case("r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 2 2", 1, 19; "misc 3 depth 1")]
    #[test_case("r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQkq - 3 2", 1, 5; "misc 4 depth 1")]
    #[test_case("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 3, 62379; "misc 5 depth 3")]
    #[test_case("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 3, 89890; "misc 6 depth 3")]
    fn perft_tests(position_fen: &str, depth: u32, expected_result: u64) {
        assert_eq!(
            Board::from_str(position_fen).unwrap().perft(depth),
            expected_result
        );
    }
}

#![deny(unsafe_op_in_unsafe_fn)]

mod board;
mod cache;
mod game;
mod index;
mod mg;
mod repr;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::game::Game;

    #[test]
    fn fen_default() {
        assert_eq!(
            String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
            Game::default().to_string()
        );
    }

    #[test]
    fn perft_default_1() {
        assert_eq!(Game::default().perft(3), 8902);
    }

    #[test]
    fn perft_default_2() {
        assert_eq!(Game::default().perft(5), 4865609);
    }

    #[test]
    fn perft_default_3() {
        assert_eq!(Game::default().perft(6), 119060324);
    }

    #[test]
    fn perft_default_4() {
        assert_eq!(Game::default().perft(7), 3195901860);
    }

    #[test]
    fn perft_kiwipete_1() {
        assert_eq!(
            Game::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .unwrap()
                .perft(1),
            48
        );
    }

    #[test]
    fn perft_kiwipete_2() {
        assert_eq!(
            Game::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .unwrap()
                .perft(4),
            4085603
        )
    }

    #[test]
    fn perft_kiwipete_3() {
        assert_eq!(
            Game::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .unwrap()
                .perft(5),
            193690690
        )
    }

    #[test]
    fn perft_misc_1() {
        assert_eq!(
            Game::from_str("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2")
                .unwrap()
                .perft(1),
            8
        );
    }

    #[test]
    fn perft_misc_2() {
        assert_eq!(
            Game::from_str("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3")
                .unwrap()
                .perft(1),
            8
        );
    }

    #[test]
    fn perft_misc_3() {
        assert_eq!(
            Game::from_str("r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 2 2")
                .unwrap()
                .perft(1),
            19
        );
    }

    #[test]
    fn perft_misc_4() {
        assert_eq!(
            Game::from_str("r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQkq - 3 2")
                .unwrap()
                .perft(1),
            5
        );
    }

    #[test]
    fn perft_misc_5() {
        assert_eq!(
            Game::from_str("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                .unwrap()
                .perft(3),
            62379
        );
    }

    #[test]
    fn perft_misc_6() {
        assert_eq!(
            Game::from_str(
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
            )
                .unwrap()
                .perft(3),
            89890
        );
    }

    #[test]
    fn perft_misc_7() {
        assert_eq!(
            Game::from_str("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1")
                .unwrap()
                .perft(6),
            1134888
        );
    }
}

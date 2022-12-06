use arrayvec::ArrayVec;
use before_build::{BitBoard, Orientation, Square};

use crate::{
    index,
    repr::{Board, Color, EpData, Move, PieceKind},
};

type Moves = ArrayVec<Move, 218>;

pub trait Gen {
    fn dangers(
        pieces: BitBoard,
        occupation: BitBoard,
        orientation: Orientation,
        dangers: &mut BitBoard,
    );

    fn legal_moves(
        board: &Board,
        occupation: BitBoard,
        orientation: Orientation,
        moves: &mut Moves,
    );
}

pub struct Pawn;

impl Pawn {
    fn is_legal_ep_capture(
        board: &Board,
        mut occupation: BitBoard,
        ep_data: EpData,
        origin: Square,
    ) -> bool {
        // Update board to it's post capture state
        occupation.toggle_bit(origin);
        occupation.toggle_bit(ep_data.capture_point.first_one_as_square());
        occupation.toggle_bit(ep_data.pawn);

        let king = board.current_player.king.first_one_as_square();

        // Test for any rays hitting the king
        (index::bishop_slides(king, occupation) + index::rook_slides(king, occupation)
            & (board.opposing_player.queens
                + board.opposing_player.rooks
                + board.opposing_player.bishops))
            .is_empty()
    }
}

impl Gen for Pawn {
    fn dangers(
        pieces: BitBoard,
        _occupation: BitBoard,
        orientation: Orientation,
        dangers: &mut BitBoard,
    ) {
        *dangers += pieces.move_one_up_left(orientation) + pieces.move_one_up_right(orientation);
    }

    // TODO: There is some debate in the CP community over whether set-wise or piece-wise
    // operations are better here. Currently set-wise is used for familiarity and hopefully speed.
    // I want to verfiy what option is truely the best here.
    fn legal_moves(
        board: &Board,
        occupation: BitBoard,
        orientation: Orientation,
        moves: &mut Moves,
    ) {
        let unpinned_push_pawns =
            board.current_player.pawns & board.current_player.pins.vertical_movement();

        // Pawn pushes
        {
            let pawn_targets = (unpinned_push_pawns.move_one_up(orientation)
                & board.current_player.valid_targets)
                - occupation;

            for target in pawn_targets {
                if target.rank() == 0 || target.rank() == 7 {
                    moves.extend(PieceKind::PROMOTIONS.into_iter().map(|piece| {
                        Move::Promotion {
                            // SAFETY: Reversing the movement always returns a valid pawn square, by definition
                            origin: unsafe { target.move_one_down_unchecked(orientation) },
                            target,
                            to: piece,
                        }
                    }));
                } else {
                    moves.push(Move::Simple {
                        // SAFETY: Reversing the movement always returns a valid pawn square, by definition
                        origin: unsafe { target.move_one_down_unchecked(orientation) },
                        target,
                        is_double_push: false,
                    })
                }
            }
        }

        // Pawn double pushes
        {
            let pawn_targets = ((unpinned_push_pawns & BitBoard::PAWN_START_RANKS)
                .move_two_up(orientation)
                & board.current_player.valid_targets)
                // The smearing blocks pushes through pieces
                - occupation.smear_ones_up(orientation);

            moves.extend(pawn_targets.bits().map(|target| Move::Simple {
                // SAFETY: See above
                origin: unsafe { target.move_two_down_unchecked(orientation) },
                target,
                is_double_push: true,
            }));
        }

        let unpinned_right_capture_pawns =
            board.current_player.pawns & board.current_player.pins.diagonal_movement();

        // Right pawn captures
        {
            let pawn_targets = unpinned_right_capture_pawns.move_one_up_right(orientation)
                & board.current_player.valid_targets
                & board.opposing_player.occupation;

            moves.extend(pawn_targets.bits().map(|target| Move::Simple {
                // SAFETY: See above
                origin: unsafe { target.move_one_down_left_unchecked(orientation) },
                target,
                is_double_push: false,
            }));
        }

        let unpinned_left_capture_pawns =
            board.current_player.pawns & board.current_player.pins.anti_diagonal_movement();

        // Left pawn captures
        {
            let pawn_targets = unpinned_left_capture_pawns.move_one_up_left(orientation)
                & board.current_player.valid_targets
                & board.opposing_player.occupation;

            moves.extend(pawn_targets.bits().map(|target| Move::Simple {
                // SAFETY: See above
                origin: unsafe { target.move_one_down_right_unchecked(orientation) },
                target,
                is_double_push: false,
            }));
        }

        if let Some(ep_data @ EpData { capture_point, .. }) = board.ep_data {
            if (capture_point & board.current_player.valid_targets).isnt_empty() {
                // Right EP captures
                {
                    let capturer = capture_point.move_one_down_left(orientation)
                        & unpinned_right_capture_pawns;

                    if capturer.isnt_empty()
                        && Pawn::is_legal_ep_capture(
                            &board,
                            occupation,
                            ep_data,
                            capturer.first_one_as_square(),
                        )
                    {
                        moves.push(Move::EnPassant {
                            origin: capturer.first_one_as_square(),
                            target: capture_point.first_one_as_square(),
                        });
                    }
                }

                // Left EP captures
                {
                    let capturer = capture_point.move_one_down_right(orientation)
                        & unpinned_left_capture_pawns;

                    if capturer.isnt_empty()
                        && Pawn::is_legal_ep_capture(
                            &board,
                            occupation,
                            ep_data,
                            capturer.first_one_as_square(),
                        )
                    {
                        moves.push(Move::EnPassant {
                            origin: capturer.first_one_as_square(),
                            target: capture_point.first_one_as_square(),
                        });
                    }
                }
            }
        }
    }
}

pub struct Knight;

impl Gen for Knight {
    fn dangers(
        pieces: BitBoard,
        _occupation: BitBoard,
        _orientation: Orientation,
        dangers: &mut BitBoard,
    ) {
        for piece in pieces {
            *dangers += index::knight_attacks(piece);
        }
    }

    fn legal_moves(
        board: &Board,
        _occupation: BitBoard,
        _orientation: Orientation,
        moves: &mut Moves,
    ) {
        for piece in board.current_player.knights - board.current_player.pins.all() {
            let attacks = (index::knight_attacks(piece) & board.current_player.valid_targets)
                - board.current_player.occupation;

            moves.extend(attacks.bits().map(|target| Move::Simple {
                origin: piece,
                target,
                is_double_push: false,
            }))
        }
    }
}

pub struct Bishop;

impl Gen for Bishop {
    fn dangers(
        pieces: BitBoard,
        occupation: BitBoard,
        _orientation: Orientation,
        dangers: &mut BitBoard,
    ) {
        for piece in pieces {
            *dangers += index::bishop_slides(piece, occupation);
        }
    }

    fn legal_moves(
        board: &Board,
        occupation: BitBoard,
        _orientation: Orientation,
        moves: &mut Moves,
    ) {
        let valid_bishops = board.current_player.bishops - board.current_player.pins.cross_pins();

        // One for unpinned bishops
        for bishop in valid_bishops - board.current_player.pins.diagonal_pins() {
            moves.extend(
                (index::bishop_slides(bishop, occupation) - board.current_player.occupation)
                    .bits()
                    .map(|target| Move::Simple {
                        origin: bishop,
                        target,
                        is_double_push: false,
                    }),
            );
        }

        // One for pinned bishops
        for bishop in valid_bishops & board.current_player.pins.diagonal_pins() {
            moves.extend(
                ((index::bishop_slides(bishop, occupation)
                    & board.current_player.pins.diagonal_pins())
                    - board.current_player.valid_targets)
                    .bits()
                    .map(|target| Move::Simple {
                        origin: bishop,
                        target,
                        is_double_push: false,
                    }),
            );
        }
    }
}

pub struct Rook;

impl Gen for Rook {
    fn dangers(
        pieces: BitBoard,
        occupation: BitBoard,
        _orientation: Orientation,
        dangers: &mut BitBoard,
    ) {
        for piece in pieces {
            *dangers += index::rook_slides(piece, occupation);
        }
    }

    fn legal_moves(
        board: &Board,
        occupation: BitBoard,
        _orientation: Orientation,
        moves: &mut Moves,
    ) {
        let valid_rooks = board.current_player.rooks - board.current_player.pins.diagonal_pins();

        // One for unpinned rooks
        for rook in valid_rooks - board.current_player.pins.cross_pins() {
            moves.extend(
                (index::rook_slides(rook, occupation) - board.current_player.occupation)
                    .bits()
                    .map(|target| Move::Simple {
                        origin: rook,
                        target,
                        is_double_push: false,
                    }),
            );
        }

        // One for pinned rooks
        for rook in valid_rooks & board.current_player.pins.cross_pins() {
            moves.extend(
                ((index::rook_slides(rook, occupation) & board.current_player.pins.cross_pins())
                    - board.current_player.valid_targets)
                    .bits()
                    .map(|target| Move::Simple {
                        origin: rook,
                        target,
                        is_double_push: false,
                    }),
            );
        }
    }
}

pub struct Queen;

impl Gen for Queen {
    fn dangers(
        pieces: BitBoard,
        occupation: BitBoard,
        _orientation: Orientation,
        dangers: &mut BitBoard,
    ) {
        for piece in pieces {
            *dangers +=
                index::rook_slides(piece, occupation) + index::bishop_slides(piece, occupation);
        }
    }

    fn legal_moves(
        board: &Board,
        occupation: BitBoard,
        _orientation: Orientation,
        moves: &mut Moves,
    ) {
        // One for cross-pinned queens
        for queen in board.current_player.queens & board.current_player.pins.cross_pins() {
            moves.extend(
                ((index::rook_slides(queen, occupation) - board.current_player.occupation)
                    & board.current_player.valid_targets
                    & board.current_player.pins.cross_pins())
                .bits()
                .map(|target| Move::Simple {
                    origin: queen,
                    target,
                    is_double_push: false,
                }),
            );
        }

        // One for diagonally-pinned queens
        for queen in board.current_player.queens & board.current_player.pins.diagonal_pins() {
            moves.extend(
                ((index::bishop_slides(queen, occupation) - board.current_player.occupation)
                    & board.current_player.valid_targets
                    & board.current_player.pins.diagonal_pins())
                .bits()
                .map(|target| Move::Simple {
                    origin: queen,
                    target,
                    is_double_push: false,
                }),
            );
        }

        // And one for unpinned queens
        for queen in board.current_player.queens - board.current_player.pins.all() {
            moves.extend(
                ((index::rook_slides(queen, occupation) + index::bishop_slides(queen, occupation)
                    - board.current_player.occupation)
                    & board.current_player.valid_targets)
                    .bits()
                    .map(|target| Move::Simple {
                        origin: queen,
                        target,
                        is_double_push: false,
                    }),
            );
        }
    }
}

pub struct King;

impl Gen for King {
    fn dangers(
        pieces: BitBoard,
        _occupation: BitBoard,
        _orientation: Orientation,
        dangers: &mut BitBoard,
    ) {
        *dangers += index::king_attacks(pieces.first_one_as_square());
    }

    fn legal_moves(
        board: &Board,
        occupation: BitBoard,
        orientation: Orientation,
        moves: &mut Moves,
    ) {
        // Non castles
        {
            let origin = board.current_player.king.first_one_as_square();

            moves.extend(
                (index::king_attacks(origin)
                    - board.current_player.dangers
                    - board.current_player.occupation)
                    .bits()
                    .map(|target| Move::Simple {
                        origin,
                        target,
                        is_double_push: false,
                    }),
            );
        }

        // Castles
        {
            if board.current_player.can_castle_ks
                && (BitBoard::ks_space(orientation) & (occupation + board.current_player.dangers))
                    .is_empty()
            {
                moves.push(Move::CastleKs);
            }

            if board.current_player.can_castle_qs
                && (BitBoard::qs_move_space(orientation) & occupation).is_empty()
                && (BitBoard::qs_danger_space(orientation) & board.current_player.dangers)
                    .is_empty()
            {
                moves.push(Move::CastleQs);
            }
        }
    }
}

fn gen_dangers(board: &mut Board) {
    let occupation = board.current_player.occupation + board.opposing_player.occupation;
    let orientation = match board.playing_side {
        Color::White => Orientation::TopToBottom,
        Color::Black => Orientation::BottomToTop,
    };

    Pawn::dangers(
        board.opposing_player.pawns,
        occupation,
        orientation,
        &mut board.current_player.dangers,
    );
    Knight::dangers(
        board.opposing_player.knights,
        occupation,
        orientation,
        &mut board.current_player.dangers,
    );
    Bishop::dangers(
        board.opposing_player.bishops,
        occupation,
        orientation,
        &mut board.current_player.dangers,
    );
    Rook::dangers(
        board.opposing_player.rooks,
        occupation,
        orientation,
        &mut board.current_player.dangers,
    );
    Queen::dangers(
        board.opposing_player.queens,
        occupation,
        orientation,
        &mut board.current_player.dangers,
    );
    King::dangers(
        board.opposing_player.king,
        occupation,
        orientation,
        &mut board.current_player.dangers,
    );
}

fn gen_moves(board: &Board) -> Moves {
    let mut moves = Moves::new();
    let occupation = board.current_player.occupation + board.opposing_player.occupation;
    let orientation = match board.playing_side {
        Color::White => Orientation::BottomToTop,
        Color::Black => Orientation::TopToBottom,
    };

    if !board.current_player.king_must_move {
        Pawn::legal_moves(&board, occupation, orientation, &mut moves);
        Knight::legal_moves(&board, occupation, orientation, &mut moves);
        Bishop::legal_moves(&board, occupation, orientation, &mut moves);
        Rook::legal_moves(&board, occupation, orientation, &mut moves);
        Queen::legal_moves(&board, occupation, orientation, &mut moves);
    }

    King::legal_moves(&board, occupation, orientation, &mut moves);

    moves
}

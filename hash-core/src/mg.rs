use arrayvec::ArrayVec;
use hash_build::{BitBoard, Color, Square};

use crate::{
    board::Board,
    index,
    repr::{EpData, Move, MoveMeta, PieceKind},
};

pub const MOVES: usize = 218;
pub type Moves = ArrayVec<Move, MOVES>;

pub trait Gen {
    fn dangers(pieces: BitBoard, occupation: BitBoard, color: Color, dangers: &mut BitBoard);

    fn legal_moves(board: &Board, occupation: BitBoard, moves: &mut Moves);
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
        ((index::bishop_slides(king, occupation)
            & (board.opposing_player.queens + board.opposing_player.bishops))
            + (index::rook_slides(king, occupation)
                & (board.opposing_player.queens + board.opposing_player.rooks)))
            .is_empty()
    }

    fn update_moves(origin: Square, target: Square, moves: &mut Moves) {
        if target.rank() == 0 || target.rank() == 7 {
            moves.extend(PieceKind::PROMOTIONS.into_iter().map(|piece| {
                Move {
                    // SAFETY: Reversing the movement always returns a valid pawn square, by definition
                    origin,
                    target,
                    meta: MoveMeta::Promotion(piece),
                    moved_piece_kind: PieceKind::Pawn,
                }
            }));
        } else {
            moves.push(Move {
                // SAFETY: Reversing the movement always returns a valid pawn square, by definition
                origin,
                target,
                moved_piece_kind: PieceKind::Pawn,
                meta: MoveMeta::None,
            })
        }
    }
}

impl Gen for Pawn {
    fn dangers(pieces: BitBoard, _occupation: BitBoard, color: Color, dangers: &mut BitBoard) {
        *dangers += pieces.move_one_up_left(color) + pieces.move_one_up_right(color);
    }

    // TODO: There is some debate in the CP community over whether set-wise or piece-wise
    // operations are better here. Currently set-wise is used for familiarity and hopefully speed.
    // I want to verfiy what option is truely the best here.
    fn legal_moves(board: &Board, occupation: BitBoard, moves: &mut Moves) {
        let unpinned_push_pawns =
            board.current_player.pawns & board.current_player.pins.vertical_movement();

        // Pawn pushes
        {
            let pawn_targets = (unpinned_push_pawns.move_one_up(board.current_color)
                & board.current_player.valid_targets)
                - occupation;

            for target in pawn_targets {
                // SAFETY: Reversing the movement always returns a valid pawn square, by definition
                Self::update_moves(
                    unsafe { target.move_one_down_unchecked(board.current_color) },
                    target,
                    moves,
                );
            }
        }

        // Pawn double pushes
        {
            let pawn_targets = ((unpinned_push_pawns & BitBoard::PAWN_START_RANKS)
                .move_two_up(board.current_color)
                & board.current_player.valid_targets)
                // The smearing blocks pushes through pieces
                - occupation.smear_ones_up(board.current_color);

            moves.extend(pawn_targets.bits().map(|target| Move {
                // SAFETY: See above
                origin: unsafe { target.move_two_down_unchecked(board.current_color) },
                target,
                moved_piece_kind: PieceKind::Pawn,
                meta: MoveMeta::DoublePush,
            }));
        }

        let unpinned_right_capture_pawns =
            board.current_player.pawns & board.current_player.pins.diagonal_movement();

        // Right pawn captures
        {
            let pawn_targets = unpinned_right_capture_pawns.move_one_up_right(board.current_color)
                & board.current_player.valid_targets
                & board.opposing_player.occupation;

            for target in pawn_targets {
                // SAFETY: Reversing the movement always returns a valid pawn square, by definition
                Self::update_moves(
                    unsafe { target.move_one_down_left_unchecked(board.current_color) },
                    target,
                    moves,
                );
            }
        }

        let unpinned_left_capture_pawns =
            board.current_player.pawns & board.current_player.pins.anti_diagonal_movement();

        // Left pawn captures
        {
            let pawn_targets = unpinned_left_capture_pawns.move_one_up_left(board.current_color)
                & board.current_player.valid_targets
                & board.opposing_player.occupation;

            for target in pawn_targets {
                // SAFETY: Reversing the movement always returns a valid pawn square, by definition
                Self::update_moves(
                    unsafe { target.move_one_down_right_unchecked(board.current_color) },
                    target,
                    moves,
                );
            }
        }

        if let Some(
            ep_data @ EpData {
                capture_point,
                pawn,
            },
        ) = board.ep_data
        {
            if (pawn.as_bitboard() & board.current_player.valid_targets).isnt_empty() {
                // Right EP captures
                {
                    let capturer = capture_point.move_one_down_left(board.current_color)
                        & unpinned_right_capture_pawns;

                    if capturer.isnt_empty()
                        && Pawn::is_legal_ep_capture(
                            board,
                            occupation,
                            ep_data,
                            capturer.first_one_as_square(),
                        )
                    {
                        moves.push(Move {
                            origin: capturer.first_one_as_square(),
                            moved_piece_kind: PieceKind::Pawn,
                            target: capture_point.first_one_as_square(),
                            meta: MoveMeta::EnPassant,
                        });
                    }
                }

                // Left EP captures
                {
                    let capturer = capture_point.move_one_down_right(board.current_color)
                        & unpinned_left_capture_pawns;

                    if capturer.isnt_empty()
                        && Pawn::is_legal_ep_capture(
                            board,
                            occupation,
                            ep_data,
                            capturer.first_one_as_square(),
                        )
                    {
                        moves.push(Move {
                            origin: capturer.first_one_as_square(),
                            moved_piece_kind: PieceKind::Pawn,
                            target: capture_point.first_one_as_square(),
                            meta: MoveMeta::EnPassant,
                        });
                    }
                }
            }
        }
    }
}

pub struct Knight;

impl Gen for Knight {
    fn dangers(pieces: BitBoard, _occupation: BitBoard, _color: Color, dangers: &mut BitBoard) {
        for piece in pieces {
            *dangers += index::knight_attacks(piece);
        }
    }

    fn legal_moves(board: &Board, _occupation: BitBoard, moves: &mut Moves) {
        for piece in board.current_player.knights - board.current_player.pins.all() {
            let attacks = (index::knight_attacks(piece) & board.current_player.valid_targets)
                - board.current_player.occupation;

            moves.extend(attacks.bits().map(|target| Move {
                origin: piece,
                target,
                moved_piece_kind: PieceKind::Knight,
                meta: MoveMeta::None,
            }))
        }
    }
}

pub struct Bishop;

impl Gen for Bishop {
    #[inline(always)]
    fn dangers(pieces: BitBoard, occupation: BitBoard, _color: Color, dangers: &mut BitBoard) {
        for piece in pieces {
            *dangers += index::bishop_slides(piece, occupation);
        }
    }

    #[inline(always)]
    fn legal_moves(board: &Board, occupation: BitBoard, moves: &mut Moves) {
        let valid_bishops = board.current_player.bishops - board.current_player.pins.cross_pins();

        // One for unpinned bishops
        for bishop in valid_bishops - board.current_player.pins.diagonal_pins() {
            moves.extend(
                ((index::bishop_slides(bishop, occupation) - board.current_player.occupation)
                    & board.current_player.valid_targets)
                    .bits()
                    .map(|target| Move {
                        origin: bishop,
                        target,
                        meta: MoveMeta::None,
                        moved_piece_kind: PieceKind::Bishop,
                    }),
            );
        }

        // One for pinned bishops
        for bishop in valid_bishops & board.current_player.pins.diagonal_pins() {
            moves.extend(
                (index::bishop_slides(bishop, occupation)
                    & board.current_player.pins.diagonal_pins()
                    & board.current_player.valid_targets)
                    .bits()
                    .map(|target| Move {
                        origin: bishop,
                        target,
                        meta: MoveMeta::None,
                        moved_piece_kind: PieceKind::Bishop,
                    }),
            );
        }
    }
}

pub struct Rook;

impl Gen for Rook {
    fn dangers(pieces: BitBoard, occupation: BitBoard, _color: Color, dangers: &mut BitBoard) {
        for piece in pieces {
            *dangers += index::rook_slides(piece, occupation);
        }
    }

    fn legal_moves(board: &Board, occupation: BitBoard, moves: &mut Moves) {
        let valid_rooks = board.current_player.rooks - board.current_player.pins.diagonal_pins();

        // One for unpinned rooks
        for rook in valid_rooks - board.current_player.pins.cross_pins() {
            moves.extend(
                ((index::rook_slides(rook, occupation) - board.current_player.occupation)
                    & board.current_player.valid_targets)
                    .bits()
                    .map(|target| Move {
                        origin: rook,
                        target,
                        meta: MoveMeta::None,
                        moved_piece_kind: PieceKind::Rook,
                    }),
            );
        }

        // One for pinned rooks
        for rook in valid_rooks & board.current_player.pins.cross_pins() {
            moves.extend(
                ((index::rook_slides(rook, occupation) & board.current_player.pins.cross_pins())
                    & board.current_player.valid_targets)
                    .bits()
                    .map(|target| Move {
                        origin: rook,
                        target,
                        meta: MoveMeta::None,
                        moved_piece_kind: PieceKind::Rook,
                    }),
            );
        }
    }
}

pub struct Queen;

impl Gen for Queen {
    fn dangers(pieces: BitBoard, occupation: BitBoard, _color: Color, dangers: &mut BitBoard) {
        for piece in pieces {
            *dangers +=
                index::rook_slides(piece, occupation) + index::bishop_slides(piece, occupation);
        }
    }

    fn legal_moves(board: &Board, occupation: BitBoard, moves: &mut Moves) {
        // One for cross-pinned queens
        for queen in board.current_player.queens & board.current_player.pins.cross_pins() {
            moves.extend(
                ((index::rook_slides(queen, occupation) - board.current_player.occupation)
                    & board.current_player.valid_targets
                    & board.current_player.pins.cross_pins())
                .bits()
                .map(|target| Move {
                    origin: queen,
                    target,
                    meta: MoveMeta::None,
                    moved_piece_kind: PieceKind::Queen,
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
                .map(|target| Move {
                    origin: queen,
                    target,
                    meta: MoveMeta::None,
                    moved_piece_kind: PieceKind::Queen,
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
                    .map(|target| Move {
                        origin: queen,
                        target,
                        meta: MoveMeta::None,
                        moved_piece_kind: PieceKind::Queen,
                    }),
            );
        }
    }
}

pub struct King;

impl Gen for King {
    fn dangers(pieces: BitBoard, _occupation: BitBoard, _color: Color, dangers: &mut BitBoard) {
        *dangers += index::king_attacks(pieces.first_one_as_square());
    }

    fn legal_moves(board: &Board, occupation: BitBoard, moves: &mut Moves) {
        let origin = board.current_player.king.first_one_as_square();
        // Non castles
        {
            moves.extend(
                (index::king_attacks(origin)
                    - board.current_player.dangers
                    - board.current_player.occupation)
                    .bits()
                    .map(|target| Move {
                        origin,
                        target,
                        meta: MoveMeta::None,
                        moved_piece_kind: PieceKind::King,
                    }),
            );
        }

        // Castles
        if !board.current_player.is_in_check() {
            if board.current_player.castling_rights.can_castle_ks()
                && (BitBoard::ks_space(board.current_color)
                    & (occupation + board.current_player.dangers))
                    .is_empty()
            {
                moves.push(Move {
                    origin,
                    target: match board.current_color {
                        Color::White => Square::G1,
                        Color::Black => Square::G8,
                    },
                    moved_piece_kind: PieceKind::King,
                    meta: MoveMeta::CastleKs,
                });
            }

            if board.current_player.castling_rights.can_castle_qs()
                && (BitBoard::qs_move_space(board.current_color) & occupation).is_empty()
                && (BitBoard::qs_danger_space(board.current_color) & board.current_player.dangers)
                    .is_empty()
            {
                moves.push(Move {
                    origin,
                    target: match board.current_color {
                        Color::White => Square::C1,
                        Color::Black => Square::C8,
                    },
                    moved_piece_kind: PieceKind::King,
                    meta: MoveMeta::CastleQs,
                });
            }
        }
    }
}

pub fn gen_dangers(board: &mut Board) {
    board.current_player.dangers = BitBoard::EMPTY;
    let occupation = (board.current_player.occupation + board.opposing_player.occupation)
        - board.current_player.king;
    let color = !board.current_color;

    Pawn::dangers(
        board.opposing_player.pawns,
        occupation,
        color,
        &mut board.current_player.dangers,
    );
    Knight::dangers(
        board.opposing_player.knights,
        occupation,
        color,
        &mut board.current_player.dangers,
    );
    Bishop::dangers(
        board.opposing_player.bishops,
        occupation,
        color,
        &mut board.current_player.dangers,
    );
    Rook::dangers(
        board.opposing_player.rooks,
        occupation,
        color,
        &mut board.current_player.dangers,
    );
    Queen::dangers(
        board.opposing_player.queens,
        occupation,
        color,
        &mut board.current_player.dangers,
    );
    King::dangers(
        board.opposing_player.king,
        occupation,
        color,
        &mut board.current_player.dangers,
    );
}

pub fn gen_moves(board: &Board) -> Moves {
    let mut moves = Moves::new();
    let occupation = board.current_player.occupation + board.opposing_player.occupation;

    if !board.current_player.king_must_move {
        Pawn::legal_moves(board, occupation, &mut moves);
        Knight::legal_moves(board, occupation, &mut moves);
        Bishop::legal_moves(board, occupation, &mut moves);
        Rook::legal_moves(board, occupation, &mut moves);
        Queen::legal_moves(board, occupation, &mut moves);
    }

    King::legal_moves(board, occupation, &mut moves);

    moves
}

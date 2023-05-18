use crate::{
    chess_move::*,
    piece::{PieceKind::*, *},
    square::*,
};

use std::collections::HashSet;
use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub struct State {
    pub squares: [Square; 64],
    pub turn: ChessColour,
    pub selected_square: Option<(u8, u8)>,
    pub mouse_pressed_previous: bool,
    pub game_running: bool,
    pub history: Vec<PerformedMove>,
    pub next_promotor: PieceKind,
}

impl State {
    pub fn new() -> Self {
        let mut squares = [Square::new(); 64];
        for y in 0..8u8 {
            for x in 0..8u8 {
                let idx = (x + 8 * y) as usize;
                squares[idx].coords = (x, y);
                squares[idx].content = Piece::from_char(match (x, y) {
                    (_, 1) => 'P',
                    (_, 6) => 'p',

                    (0, 0) | (7, 0) => 'R',
                    (0, 7) | (7, 7) => 'r',

                    (1, 0) | (6, 0) => 'N',
                    (1, 7) | (6, 7) => 'n',

                    (2, 0) | (5, 0) => 'B',
                    (2, 7) | (5, 7) => 'b',

                    (3, 0) => 'Q',
                    (3, 7) => 'q',

                    (4, 0) => 'K',
                    (4, 7) => 'k',

                    _ => ' ', // intentionally give an invalid character to the from_char function
                });
            }
        }

        State {
            squares,
            turn: ChessColour::White,
            selected_square: None,
            mouse_pressed_previous: false,
            game_running: true,
            history: Vec::new(),
            next_promotor: Queen,
        }
    }

    pub fn get_king_coord(&self, col: ChessColour) -> (u8, u8) {
        self.squares
            .into_iter()
            .find(|s| {
                if let Some(p) = s.content {
                    p.kind == PieceKind::King && p.colour == col
                } else {
                    false
                }
            })
            .expect(&format!("{:?}", self.squares))
            .coords
    }

    pub fn is_in_check(&self, col: ChessColour) -> bool {
        let king_coord = self.get_king_coord(col);
        for enemy_piece in self
            .squares
            .into_iter()
            .filter(|s| s.content.is_some())
            .map(|s| s.coords)
        {
            let enemy_moves = self
                .get_moves(enemy_piece, false)
                .into_iter()
                .map(|m| m.dst)
                .collect::<Vec<_>>();
            if enemy_moves.contains(&king_coord) {
                return true;
            }
        }

        false
    }

    // assumes the necessary checks have been performed
    pub fn perform_castle(&mut self, long_castle: bool, col: ChessColour) {
        let rook_coord: (u8, u8) = (
            if long_castle { 0 } else { 7 },
            if col == ChessColour::White { 0 } else { 7 },
        );
        let king_coord = self.get_king_coord(col);

        let rook_target: (u8, u8) = (if long_castle { 3 } else { 5 }, rook_coord.1);
        let king_target: (u8, u8) = (if long_castle { 2 } else { 6 }, king_coord.1);

        self.make_move(rook_coord, ChessMove::dummy(rook_target));
        self.make_move(king_coord, ChessMove::dummy(king_target));
    }

    pub fn make_move(&mut self, src: (u8, u8), mut chess_move: ChessMove) {
        if !(chess_move.function)(self) {
            // if executing the move's function didn't already handle piece movement for us,
            // it has to be done "manually" like this:
            let dst = chess_move.dst;
            self[dst].content = Some(Piece {
                has_moved: true,
                ..self[src].content.unwrap()
            });
            self[src].content = None;
        };
    }

    // returns whether or not the move was correctly carried out
    pub fn attempt_move(&mut self, target_coordinate: (u8, u8)) -> bool {
        //TODO: implement this properly
        let source_coordinate = self.selected_square.unwrap();
        let source_piece = self[source_coordinate].content.unwrap();
        let target_square = self[target_coordinate];

        let mut return_value = false;

        if source_coordinate != target_coordinate {
            if !(target_square.content.is_some()
                && target_square.content.unwrap().colour == source_piece.colour)
            {
                let moves = self.get_moves(source_coordinate, true);
                //TODO: rewrite
                if moves.contains(&ChessMove::dummy(target_coordinate)) {
                    let move_to_be_made = moves
                        .into_iter()
                        .filter(|m| *m == target_coordinate)
                        .next()
                        .unwrap();
                    self.make_move(source_coordinate, move_to_be_made);
                    return_value = true;
                }
            }
        }
        self.selected_square = None;

        return_value
    }

    // assumes the move's availability checks have been performed properly
    pub fn promote_pawn(&mut self, src: (u8, u8), dst: (u8, u8)) {
        match self.next_promotor {
            Pawn | King => unreachable!(),
            _ => {
                self[dst].content = Some(Piece {
                    kind: self.next_promotor,
                    colour: self[src].content.unwrap().colour,
                    has_moved: true,
                    en_passanteable: false,
                });
                self[src].content = None;
            }
        }
    }

    pub fn en_passant(&mut self, src: (u8, u8), dst: (u8, u8)) {
        self[dst].content = self[src].content;
        self[(dst.0, src.1)].content = None;
        self[src].content = None;
    }

    pub fn get_moves(&self, coord: (u8, u8), test_for_checks: bool) -> Vec<ChessMove> {
        //determine piece type, and possible move offsets
        let piece = self[coord].content.unwrap();

        let mut moves: HashSet<(u8, u8)> = HashSet::new();

        // for moves whose function we already know during move calculation
        let mut moves_with_fn = Vec::new();
        let boxed_coord = Box::new(coord);
        let static_coord: &'static (u8, u8) = Box::<(u8, u8)>::leak(boxed_coord);

        if piece.kind.is_sliding() {
            let offsets: &[(i8, i8)] = match piece.kind {
                Queen => &QUEEN_OFFSETS,
                Rook => &ROOK_OFFSETS,
                Bishop => &BISHOP_OFFSETS,
                _ => unreachable!("Supposed sliding piece isn't a queen, rook or bishop"),
            };

            for direction in offsets {
                let mut current_coord = coord;
                loop {
                    let next_coord = (
                        current_coord.0 as i8 + direction.0,
                        current_coord.1 as i8 + direction.1,
                    );

                    if !(0..8).contains(&next_coord.0) || !(0..8).contains(&next_coord.1) {
                        break;
                    }

                    let next_as_valid = (next_coord.0 as u8, next_coord.1 as u8);

                    if let Some(next_hit_piece) = self[next_as_valid].content {
                        if next_hit_piece.colour == piece.colour.flip() {
                            moves.insert(current_coord);
                            moves.insert(next_as_valid);
                            break;
                        } else {
                            moves.insert(current_coord);
                            break;
                        }
                    }
                    current_coord = next_as_valid;

                    moves.insert(current_coord);
                }
            }

            // sliding pieces have this problem; cba to figure out why so manually remove it
            moves.remove(&coord);
        } else {
            let offsets_raw: &[(i8, i8)] = match piece.kind {
                Pawn => {
                    if piece.colour == ChessColour::White {
                        if coord.1 == 6 {
                            &[]
                        } else {
                            if piece.has_moved {
                                &[(0, 1)]
                            } else {
                                &[(0, 1), (0, 2)]
                            }
                        }
                    } else {
                        if coord.1 == 1 {
                            &[]
                        } else {
                            if piece.has_moved {
                                &[(0, -1)]
                            } else {
                                &[(0, -1), (0, -2)]
                            }
                        }
                    }
                }
                Knight => &KNIGHT_MOVES_RAW,
                King => &KING_MOVES_RAW,
                _ => unreachable!("supposedly nonsliding piece was a rook, bishop or queen"),
            };

            moves = offsets_raw
                .into_iter()
                .map(|m| (coord.0 as i8 + m.0, coord.1 as i8 + m.1))
                .filter(|m| (0..8).contains(&m.0) && (0..8).contains(&m.1))
                .map(|m| (m.0 as u8, m.1 as u8))
                .filter(|s| {
                    self[*s].content.is_none()
                        || self[*s].content.unwrap().colour == piece.colour.flip()
                })
                .collect::<HashSet<_>>();

            if piece.kind == Pawn {
                let up_dir: i8 = if piece.colour == ChessColour::White {
                    1
                } else {
                    -1
                };

                let up_coord = (coord.0, (coord.1 as i8 + up_dir) as u8);
                let double_up_coord = (coord.0, (coord.1 as i8 + 2 * up_dir) as u8);

                // manually remove double forward capture when
                // pawn has not yet moved
                if !piece.has_moved && self[double_up_coord].content.is_some() {
                    moves.remove(&double_up_coord);
                }

                // cannot move forwards

                if (0..8).contains(&up_coord.1) {
                    if self[up_coord].content.is_some() {
                        moves.remove(&up_coord);
                    }

                    // only check for leftwards pawn captures if the pawn is not on the a file
                    if coord.0 >= 1 {
                        // following line does not cause board overflow because a pawn on 1st or 8th rank promotes
                        let target_coord = (coord.0 - 1, (coord.1 as i8 + up_dir) as u8);
                        let left_capture = self[target_coord].content;
                        if left_capture.is_some()
                            && left_capture.unwrap().colour == piece.colour.flip()
                        {
                            moves.insert(target_coord);
                        }

                        // also, leftward en passant captures!
                        if let Some(en_passant_pawn) = self[(coord.0 - 1, coord.1)].content {
                            if en_passant_pawn.en_passanteable {
                                let dst = (coord.0 - 1, (coord.1 as i8 + up_dir) as u8);

                                let boxed_dst = Box::new(dst);
                                let static_dst: &'static (u8, u8) =
                                    Box::<(u8, u8)>::leak(boxed_dst);

                                moves_with_fn.push(ChessMove {
                                    dst,
                                    function: Box::new(|state: &mut State| {
                                        state
                                            .history
                                            .push(PerformedMove::new(*static_coord, *static_dst));
                                        state.en_passant(*static_coord, *static_dst);
                                        true
                                    }),
                                })
                            }
                        }
                    }

                    // only check for rightwards pawn captures if the pawn is not on the h file
                    if coord.0 <= 6 {
                        // following line does not cause board overflow because a pawn on 1st or 8th rank promotes
                        let target_coord = (coord.0 + 1, (coord.1 as i8 + up_dir) as u8);
                        let right_capture = self[target_coord].content;
                        if right_capture.is_some()
                            && right_capture.unwrap().colour == piece.colour.flip()
                        {
                            moves.insert(target_coord);
                        }

                        // also, rightward en passant captures!
                        if let Some(en_passant_pawn) = self[(coord.0 + 1, coord.1)].content {
                            if en_passant_pawn.en_passanteable {
                                let dst = (coord.0 + 1, (coord.1 as i8 + up_dir) as u8);

                                let boxed_dst = Box::new(dst);
                                let static_dst: &'static (u8, u8) =
                                    Box::<(u8, u8)>::leak(boxed_dst);

                                moves_with_fn.push(ChessMove {
                                    dst,
                                    function: Box::new(|state: &mut State| {
                                        state
                                            .history
                                            .push(PerformedMove::new(*static_coord, *static_dst));
                                        state.en_passant(*static_coord, *static_dst);
                                        true
                                    }),
                                })
                            }
                        }
                    }
                }
            }
        }

        // we need to test if this move would cause the player to be in check
        // we do this by iterating over every piece the opponent has, and seeing if capturing the king is a possible move
        // if so, the move is invalid. sadly this process is fairly lengthy

        let nonchecking_moves: Vec<(u8, u8)> = if test_for_checks {
            moves
                .into_iter()
                .filter(|possibly_checking_move| {
                    // if piece.kind == Pawn && (possibly_checking_move.1 == 0 || possibly_checking_move.1 == 7) {
                    //     return true;
                    // }

                    let mut test_board = self.clone();
                    test_board.make_move(coord, ChessMove::dummy(*possibly_checking_move));

                    !test_board.is_in_check(piece.colour)
                })
                .collect()
        } else {
            moves.into_iter().collect()
        };

        // all hitherto calculated moves have no extra "functionality"
        // there are three main exceptions to this: en passant, castling and pawn promotion

        for m in nonchecking_moves {
            let boxed_move = Box::new(m);
            let static_move: &'static (u8, u8) = Box::<(u8, u8)>::leak(boxed_move);

            if piece.kind == Pawn && m.1 == 0 || m.1 == 7 {
                moves_with_fn.push(ChessMove {
                    dst: m,
                    function: Box::new(|state: &mut State| {
                        state
                            .history
                            .push(PerformedMove::new(*static_coord, *static_move));
                        state.promote_pawn(*static_coord, *static_move);
                        true
                    }),
                })
            } else if piece.kind == Pawn
                && (m.1 as i8 - coord.1 as i8).abs() == 2
                && !piece.has_moved
            {
                // double pawn move
                let boxed_col = Box::new(piece.colour);
                let static_col: &'static ChessColour = Box::<ChessColour>::leak(boxed_col);

                moves_with_fn.push(ChessMove {
                    dst: m,
                    function: Box::new(|state: &mut State| {
                        state
                            .history
                            .push(PerformedMove::new(*static_coord, *static_move));
                        state[*static_coord].content = Some(Piece {
                            en_passanteable: true,
                            has_moved: true,
                            colour: *static_col,
                            kind: Pawn,
                        }); // we set the en_passanteable field, then pass movement on
                        false
                    }),
                })
            } else {
                moves_with_fn.push(ChessMove {
                    dst: m,
                    function: Box::new(|state: &mut State| {
                        state
                            .history
                            .push(PerformedMove::new(*static_coord, *static_move));

                        false
                    }),
                });
            }
        }

        // potentially adding in castling

        let king_coord = self.get_king_coord(piece.colour);
        if coord == king_coord && !self[king_coord].content.unwrap().has_moved {
            // long castle
            if let Some(piece_on_a_file) = self[(0, king_coord.1)].content {
                if piece_on_a_file.kind == Rook
                    && !piece_on_a_file.has_moved
                    && self[(1, king_coord.1)].content.is_none()
                    && self[(2, king_coord.1)].content.is_none()
                    && self[(3, king_coord.1)].content.is_none()
                {
                    let target_move = (king_coord.0 - 2, king_coord.1);

                    let boxed_move = Box::new(target_move);
                    let static_move: &'static (u8, u8) = Box::<(u8, u8)>::leak(boxed_move); //TODO: optimise multiple leak calls away

                    if piece.colour == ChessColour::White {
                        moves_with_fn.push(ChessMove {
                            dst: target_move,
                            function: Box::new(|state: &mut State| {
                                state
                                    .history
                                    .push(PerformedMove::new(*static_coord, *static_move));
                                state.perform_castle(true, ChessColour::White);
                                true
                            }),
                        });
                    } else {
                        moves_with_fn.push(ChessMove {
                            dst: target_move,
                            function: Box::new(|state: &mut State| {
                                state
                                    .history
                                    .push(PerformedMove::new(*static_coord, *static_move));
                                state.perform_castle(true, ChessColour::Black);
                                true
                            }),
                        });
                    }
                }
            }

            if let Some(piece_on_h_file) = self[(7, king_coord.1)].content {
                if piece_on_h_file.kind == Rook
                    && !piece_on_h_file.has_moved
                    && self[(5, king_coord.1)].content.is_none()
                    && self[(6, king_coord.1)].content.is_none()
                {
                    let target_move = (king_coord.0 + 2, king_coord.1);

                    let boxed_move = Box::new(target_move);
                    let static_move: &'static (u8, u8) = Box::<(u8, u8)>::leak(boxed_move); //TODO: optimise multiple leak calls away

                    if piece.colour == ChessColour::White {
                        moves_with_fn.push(ChessMove {
                            dst: target_move,
                            function: Box::new(|state: &mut State| {
                                state
                                    .history
                                    .push(PerformedMove::new(*static_coord, *static_move));
                                state.perform_castle(false, ChessColour::White);
                                true
                            }),
                        });
                    } else {
                        moves_with_fn.push(ChessMove {
                            dst: target_move,
                            function: Box::new(|state: &mut State| {
                                state
                                    .history
                                    .push(PerformedMove::new(*static_coord, *static_move));
                                state.perform_castle(false, ChessColour::Black);
                                true
                            }),
                        });
                    }
                }
            }
        }

        moves_with_fn
    }
}

impl Index<(u8, u8)> for State {
    type Output = Square;
    fn index(&self, index: (u8, u8)) -> &Self::Output {
        &self.squares[(index.0 + 8 * index.1) as usize]
    }
}

impl IndexMut<(u8, u8)> for State {
    fn index_mut(&mut self, index: (u8, u8)) -> &mut Self::Output {
        &mut self.squares[(index.0 + 8 * index.1) as usize]
    }
}

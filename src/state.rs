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
            .unwrap()
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

    fn perform_castle(&mut self, long_castle: bool, col: ChessColour) {
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

    fn make_move(&mut self, src: (u8, u8), mut chess_move: ChessMove) {
        let dst = chess_move.dst;
        self[dst].content = Some(Piece {
            has_moved: true,
            ..self[src].content.unwrap()
        });
        self[src].content = None;
        (chess_move.function)(self);
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

    pub fn get_moves(&self, coord: (u8, u8), test_for_checks: bool) -> Vec<ChessMove> {
        //determine piece type, and possible move offsets
        let piece = self[coord].content.unwrap();

        let mut moves: HashSet<(u8, u8)> = HashSet::new();

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
                    if piece.has_moved {
                        &PAWN_MOVES_RAW_MOVED
                    } else {
                        &PAWN_MOVES_RAW_UNMOVED
                    }
                }
                Knight => &KNIGHT_MOVES_RAW,
                King => &KING_MOVES_RAW,
                _ => unreachable!("supposedly nonsliding piece was a rook, bishop or queen"),
            };

            moves = offsets_raw
                .into_iter()
                .map(|m| {
                    if piece.colour == ChessColour::White {
                        *m
                    } else {
                        (-m.0, -m.1)
                    }
                }) // black moves are flipped (only matters for pawns)
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

                // cannot move forwards

                let up_coord = (coord.0, (coord.1 as i8 + up_dir) as u8);

                if self[up_coord].content.is_some() {
                    moves.remove(&up_coord);
                }

                // only check for leftwards pawn captures if the pawn is not on the a file
                if coord.0 >= 1 {
                    // following line does not cause board overflow because a pawn on 1st or 8th rank promotes
                    let target_coord = (coord.0 - 1, (coord.1 as i8 + up_dir) as u8);
                    let left_capture = self[target_coord].content;
                    if left_capture.is_some() && left_capture.unwrap().colour == piece.colour.flip()
                    {
                        moves.insert(target_coord);
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

        let normal_moves = nonchecking_moves
            .iter()
            .map(|s| ChessMove {
                dst: *s,
                function: Box::new(|state: &mut State| {
                    state.history.push(PerformedMove::new(coord, *s));
                }),
            })
            .collect();

        normal_moves
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

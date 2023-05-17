use crate::State;

pub const PAWN_MOVES_RAW_UNMOVED: [(i8, i8); 2] = [(0, 1), (0, 2)];
pub const PAWN_MOVES_RAW_MOVED: [(i8, i8); 1] = [(0, 1)];
pub const KNIGHT_MOVES_RAW: [(i8, i8); 8] = [
    (1, 2),
    (-1, 2),
    (2, 1),
    (-2, 1),
    (2, -1),
    (-2, -1),
    (1, -2),
    (-1, -2),
];
pub const KING_MOVES_RAW: [(i8, i8); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, 1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
];

pub const ROOK_OFFSETS: [(i8, i8); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
pub const BISHOP_OFFSETS: [(i8, i8); 4] = [(-1, -1), (1, -1), (-1, 1), (1, 1)];
pub const QUEEN_OFFSETS: [(i8, i8); 8] = [
    (-1, -1),
    (1, -1),
    (-1, 1),
    (1, 1),
    (-1, 0),
    (1, 0),
    (0, -1),
    (0, 1),
];

pub struct ChessMove {
    pub dst: (u8, u8),
    /// returned boolean stands for if pieces were moved during the function execution
    pub function: Box<dyn FnMut(&mut State) -> bool>,
}

impl PartialEq for ChessMove {
    fn eq(&self, other: &Self) -> bool {
        self.dst.eq(&other.dst)
    }
}

impl PartialEq<(u8, u8)> for ChessMove {
    fn eq(&self, other: &(u8, u8)) -> bool {
        self.dst.eq(other)
    }
}

impl ChessMove {
    pub fn dummy(dst: (u8, u8)) -> Self {
        ChessMove {
            dst,
            function: Box::new(|_| false),
        }
    }
}

#[derive(Clone, Copy)]
pub struct PerformedMove {
    src: (u8, u8),
    dst: (u8, u8),
}

use std::fmt;

impl fmt::Display for PerformedMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            (self.src.0 + 97) as char,
            (self.src.1 + 49) as char,
            (self.dst.0 + 97) as char,
            (self.dst.1 + 49) as char
        )
    }
}

impl PerformedMove {
    pub fn new(src: (u8, u8), dst: (u8, u8)) -> Self {
        PerformedMove { src, dst }
    }
}

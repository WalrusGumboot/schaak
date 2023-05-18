#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl PieceKind {
    pub fn is_sliding(&self) -> bool {
        match self {
            PieceKind::Pawn | PieceKind::King | PieceKind::Knight => false,
            _ => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChessColour {
    White,
    Black,
}
impl ChessColour {
    pub fn flip(&self) -> Self {
        if self == &ChessColour::White {
            ChessColour::Black
        } else {
            ChessColour::White
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub kind: PieceKind,
    pub colour: ChessColour,
    pub has_moved: bool,
    pub en_passanteable: bool,
}

impl Piece {
    pub fn from_char(c: char) -> Option<Self> {
        let colour = if c.is_uppercase() {
            ChessColour::White
        } else {
            ChessColour::Black
        };
        let kind = match c.to_ascii_lowercase() {
            'p' => Some(PieceKind::Pawn),
            'r' => Some(PieceKind::Rook),
            'n' => Some(PieceKind::Knight),
            'b' => Some(PieceKind::Bishop),
            'q' => Some(PieceKind::Queen),
            'k' => Some(PieceKind::King),
            _ => None,
        };

        match kind {
            Some(k) => Some(Piece {
                kind: k,
                colour,
                has_moved: false,
                en_passanteable: false,
            }),
            None => None,
        }
    }
}

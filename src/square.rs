use crate::Piece;

use sdl2::pixels::Color;

#[derive(Clone, Copy)]
pub struct Square {
    pub coords: (u8, u8), // (0, 0) is a1; (7, 0) is h1; (0, 7) is a8; (7, 7) is h8
    pub content: Option<Piece>,
}

impl Square {
    pub fn new() -> Self {
        Square {
            coords: (0, 0),
            content: None,
        }
    }

    pub fn colour(&self, mouse_over: bool) -> Color {
        if (self.coords.0 + self.coords.1) % 2 == 1 {
            if mouse_over {
                Color::RGB(141, 177, 196)
            } else {
                Color::RGB(161, 222, 255)
            }
        } else {
            if mouse_over {
                Color::RGB(4, 38, 56)
            } else {
                Color::RGB(0, 79, 122)
            }
        }
    }

    pub fn coord(&self) -> String {
        let file = (self.coords.0 + 97) as char;
        let rank = (self.coords.1 + 49) as char;
        format!("{file}{rank}")
    }
}

const SQUARE_W: u32 = 60;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::image::{self, LoadTexture, InitFlag};
use sdl2::ttf::{self, Font};
use std::ops::{Index, IndexMut};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King
}

#[derive(Clone, Copy, PartialEq)]
enum ChessColour { White, Black }
impl ChessColour { fn flip(&self) -> Self { if self == &ChessColour::White { ChessColour::Black } else { ChessColour::White }}}

#[derive(Clone, Copy)]
struct Piece {
    kind: PieceKind,
    colour: ChessColour,
    has_moved: bool
}

impl Piece {
    fn from_char(c: char) -> Option<Self> {
        let colour = if c.is_uppercase() { ChessColour::White } else { ChessColour::Black };
        let kind = match c.to_ascii_lowercase() {
            'p' => Some(PieceKind::Pawn),
            'r' => Some(PieceKind::Rook),
            'n' => Some(PieceKind::Knight),
            'b' => Some(PieceKind::Bishop),
            'q' => Some(PieceKind::Queen),
            'k' => Some(PieceKind::King),
            _ => None
        };

        match kind {
            Some(k) => Some(Piece { kind: k, colour, has_moved: false }),
            None => None
        }
    }
}

#[derive(Clone, Copy)]
struct Square {
    coords: (u8, u8), // (0, 0) is a1; (7, 0) is h1; (0, 7) is a8; (7, 7) is h8
    content: Option<Piece>
}

impl Square {
    fn new() -> Self {
        Square {
            coords: (0, 0),
            content: None
        }
    }

    fn colour(&self, mouse_over: bool) -> Color {
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

    fn coord(&self) -> String {
        let file = (self.coords.0 + 97) as char;
        let rank = (self.coords.1 + 49) as char;
        format!("{file}{rank}")
    }
}

struct State {
    squares: [Square; 64],
    turn: ChessColour,
    selected_square: Option<(u8, u8)>,
    mouse_pressed_previous: bool,
}

impl State {
    fn new() -> Self {
        let mut squares = [Square::new(); 64];
        for y in 0..8u8 {
            for x in 0..8u8 {
                let idx = (x + 8 * y) as usize;
                squares[idx].coords = (x, y);
                squares[idx].content = Piece::from_char(
                    match (x, y) {
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
                    }
                );
            }
        }

        State {
            squares,
            turn: ChessColour::White,
            selected_square: None,
            mouse_pressed_previous: false
        }
    }

    // returns whether or not the move was correctly carried out
    fn attempt_move(&mut self, target_coordinate: (u8, u8)) -> bool {
        //TODO: implement this properly
        let source_coordinate = self.selected_square.unwrap();
        let source_piece = self[source_coordinate].content.unwrap();
        let target_square = self[target_coordinate];
        
        let mut return_value = false;

        if source_coordinate != target_coordinate {
            if !(target_square.content.is_some() && target_square.content.unwrap().colour == source_piece.colour) {
                self[target_coordinate].content = self[source_coordinate].content;
                self[source_coordinate].content = None;
                return_value = true;
            }
        }
        self.selected_square = None;

        return_value
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
 
fn main() {
    let mut state = State::new();

    let sdl_context = sdl2::init().unwrap();
    let image_context = image::init(InitFlag::PNG).unwrap();
    let font_context = ttf::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let font = font_context.load_font("assets/fonts/input.ttf", 16);

    let window = video_subsystem.window("schaak", SQUARE_W * 8 + 300, SQUARE_W * 8)
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();
    let tex_wp = texture_creator.load_texture("assets/textures/wp.png").unwrap();
    let tex_wr = texture_creator.load_texture("assets/textures/wr.png").unwrap();
    let tex_wn = texture_creator.load_texture("assets/textures/wn.png").unwrap();
    let tex_wb = texture_creator.load_texture("assets/textures/wb.png").unwrap();
    let tex_wq = texture_creator.load_texture("assets/textures/wq.png").unwrap();
    let tex_wk = texture_creator.load_texture("assets/textures/wk.png").unwrap();
    let tex_bp = texture_creator.load_texture("assets/textures/bp.png").unwrap();
    let tex_br = texture_creator.load_texture("assets/textures/br.png").unwrap();
    let tex_bn = texture_creator.load_texture("assets/textures/bn.png").unwrap();
    let tex_bb = texture_creator.load_texture("assets/textures/bb.png").unwrap();
    let tex_bq = texture_creator.load_texture("assets/textures/bq.png").unwrap();
    let tex_bk = texture_creator.load_texture("assets/textures/bk.png").unwrap();

 
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    
    'running: loop {
        // drawing the board tiles
        let mx = event_pump.mouse_state().x() as u32;
        let my = event_pump.mouse_state().y() as u32;
        let md = event_pump.mouse_state().is_mouse_button_pressed(sdl2::mouse::MouseButton::Left);

        if state.mouse_pressed_previous && !md { state.mouse_pressed_previous = false; }

        for y in 0..8_u8 {
            for x in 0..8_u8 {
                let square = state[(x, y)];
                let top_left_onscreen = (x as u32 * SQUARE_W, (7 - y) as u32 * SQUARE_W);

                let mouse_hit = mx >= top_left_onscreen.0 && mx < top_left_onscreen.0 + SQUARE_W &&
                my >= top_left_onscreen.1 && my < top_left_onscreen.1 + SQUARE_W;

                if mouse_hit && !state.mouse_pressed_previous && md {
                    println!("{}", square.coord());
                    if state.selected_square.is_none() {
                        if square.content.is_some() && square.content.unwrap().colour == state.turn {
                            state.selected_square = Some((x, y));
                            println!("set selected square to ({x}, {y}).")
                        }
                    } else {
                        if state.attempt_move((x, y)) {
                            state.turn = state.turn.flip();
                        }
                    }
                }
                
                let screen_rect = Rect::new(top_left_onscreen.0 as i32, top_left_onscreen.1 as i32, SQUARE_W, SQUARE_W);
                
                canvas.set_draw_color(square.colour(mouse_hit));
                if state.selected_square.is_some() && state.selected_square.unwrap() == (x, y) {
                    canvas.set_draw_color(Color::RGB(240, 200, 210));

                }
                canvas.fill_rect(screen_rect).unwrap();

                if let Some(piece) = square.content {
                    let texture = match piece {
                        Piece { kind: PieceKind::Pawn, colour: ChessColour::White, ..} => &tex_wp,
                        Piece { kind: PieceKind::Rook, colour: ChessColour::White, ..} => &tex_wr,
                        Piece { kind: PieceKind::Knight, colour: ChessColour::White, ..} => &tex_wn,
                        Piece { kind: PieceKind::Bishop, colour: ChessColour::White, ..} => &tex_wb,
                        Piece { kind: PieceKind::Queen, colour: ChessColour::White, ..} => &tex_wq,
                        Piece { kind: PieceKind::King, colour: ChessColour::White, ..} => &tex_wk,

                        Piece { kind: PieceKind::Pawn, colour: ChessColour::Black, ..} => &tex_bp,
                        Piece { kind: PieceKind::Rook, colour: ChessColour::Black, ..} => &tex_br,
                        Piece { kind: PieceKind::Knight, colour: ChessColour::Black, ..} => &tex_bn,
                        Piece { kind: PieceKind::Bishop, colour: ChessColour::Black, ..} => &tex_bb,
                        Piece { kind: PieceKind::Queen, colour: ChessColour::Black, ..} => &tex_bq,
                        Piece { kind: PieceKind::King, colour: ChessColour::Black, ..} => &tex_bk,
                    };

                    canvas.copy(texture, None, screen_rect).unwrap();
                }
            }
        }


        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

        state.mouse_pressed_previous = md;
    }
    
}

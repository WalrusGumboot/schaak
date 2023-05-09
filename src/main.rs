const SQUARE_W: u32 = 60;
const BOARD_EDGE: i32 = 8 * SQUARE_W as i32;
const MARGIN: i32 = 16; // obv only makes sense as unsigned, but this makes addition nicer
const SCREEN_W: u32 = BOARD_EDGE as u32 + 300;
const SCREEN_H: u32 = BOARD_EDGE as u32;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::image::{self, LoadTexture, InitFlag};
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::{self, Font};
use sdl2::video::{Window, WindowContext};
use std::collections::HashSet;
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

impl PieceKind {
    fn is_sliding(&self) -> bool {
        match self {
            PieceKind::Pawn | PieceKind::King | PieceKind::Knight => false,
            _ => true
        }
    }
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

const PAWN_MOVES_RAW_UNMOVED: [(i8, i8); 2] = [(0, 1), (0, 2)];
const PAWN_MOVES_RAW_MOVED:   [(i8, i8); 1] = [(0, 1)];
const KNIGHT_MOVES_RAW: [(i8, i8); 8] = [(1, 2), (-1, 2), (2, 1), (-2, 1), (2, -1), (-2, -1), (1, -2), (-1, -2)];
const KING_MOVES_RAW: [(i8, i8); 8] = [(-1, -1), (-1, 0), (-1, 1), (0, 1), (0, -1), (1, -1), (1, 0), (1, 1)];

const ROOK_OFFSETS: [(i8, i8); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
const BISHOP_OFFSETS: [(i8, i8); 4] = [(-1, -1), (1, -1), (-1, 1), (1, 1)];
const QUEEN_OFFSETS: [(i8, i8); 8] = [(-1, -1), (1, -1), (-1, 1), (1, 1), (-1, 0), (1, 0), (0, -1), (0, 1)];

fn get_moves(board: &State, coord: (u8, u8)) -> Vec<(u8, u8)> {
    use PieceKind::*;
    //determine piece type, and possible move offsets
    let piece = board[coord].content.unwrap();

    let mut moves: HashSet<(u8, u8)> = HashSet::new();
    

    if piece.kind.is_sliding() {
        let offsets: &[(i8, i8)] = match piece.kind {
            Queen => &QUEEN_OFFSETS,
            Rook => &ROOK_OFFSETS,
            Bishop => &BISHOP_OFFSETS,
            _ => unreachable!("Supposed sliding piece isn't a queen, rook or bishop")
        };

        for direction in offsets {
            let mut current_coord = coord;
            loop {
                let next_coord = (current_coord.0 as i8 + direction.0, current_coord.1 as i8 + direction.1);
                
                if !(0..8).contains(&next_coord.0) || !(0..8).contains(&next_coord.1) {
                    break;
                }

                let next_as_valid = (next_coord.0 as u8, next_coord.1 as u8);

                if let Some(next_hit_piece) = board[next_as_valid].content {
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
            Pawn => if piece.has_moved { &PAWN_MOVES_RAW_MOVED } else { &PAWN_MOVES_RAW_UNMOVED },
            Knight => &KNIGHT_MOVES_RAW,
            King => &KING_MOVES_RAW,
            _ => unreachable!("supposedly nonsliding piece was a rook, bishop or queen")
        };
    
        moves = offsets_raw.into_iter()
            .map(|m| if piece.colour == ChessColour::White { *m } else { (-m.0, -m.1) }) // black moves are flipped (only matters for pawns)
            .map(|m| (coord.0 as i8 + m.0, coord.1 as i8 + m.1))
            .filter(|m| (0..8).contains(&m.0) && (0..8).contains(&m.1))
            .map(|m| (m.0 as u8, m.1 as u8))
            .filter(|s| board[*s].content.is_none() || board[*s].content.unwrap().colour == piece.colour.flip())
            .collect::<HashSet<_>>();
    
        if piece.kind == Pawn {
            let up_dir: i8 = if piece.colour == ChessColour::White { 1 } else { -1 };
    
            // only check for leftwards pawn captures if the pawn is not on the a file
            if coord.0 >= 1 {
                // following line does not cause board overflow because a pawn on 1st or 8th rank promotes
                let target_coord = (coord.0 - 1, (coord.1 as i8 + up_dir) as u8);
                let left_capture = board[target_coord].content;
                if left_capture.is_some() && left_capture.unwrap().colour == piece.colour.flip() {
                    moves.insert(target_coord);
                }
            }
    
            // only check for rightwards pawn captures if the pawn is not on the h file
            if coord.0 <= 6 {
                // following line does not cause board overflow because a pawn on 1st or 8th rank promotes
                let target_coord = (coord.0 + 1, (coord.1 as i8 + up_dir) as u8);
                let right_capture = board[target_coord].content;
                if right_capture.is_some() && right_capture.unwrap().colour == piece.colour.flip() {
                    moves.insert(target_coord);
                }
            }
        }
    }

    // todo: en passant, castling


    moves.into_iter().collect()
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
                let moves = get_moves(&self, source_coordinate);
                
                if moves.contains(&target_coordinate) {
                    self[target_coordinate].content = Some(Piece{ has_moved: true, ..self[source_coordinate].content.unwrap() });
                    self[source_coordinate].content = None;
                    return_value = true;
                }
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

fn draw_text(text: &str, c: &mut Canvas<Window>, tc: &TextureCreator<WindowContext>, font: &Font, x: i32, y: i32) -> Result<(), String> {
    let turn_text = font.render(text);
    let text_surface = turn_text.solid(Color::WHITE).unwrap();
    let text_texture = text_surface.as_texture(tc).unwrap();

    c.copy(&text_texture, None, Rect::new(x, y, text_surface.width(), text_surface.height()))
}
 
fn main() -> Result<(), String> {
    let mut state = State::new();

    let sdl_context = sdl2::init().unwrap();
    let _image_context = image::init(InitFlag::PNG).unwrap(); // has to be let-binding to ensure drop at the end of the program
    let font_context = ttf::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let font = font_context.load_font("assets/fonts/input.ttf", 16).unwrap();

    let window = video_subsystem.window("schaak", SCREEN_W, SCREEN_H)
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
        // clear the screen
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // drawing the board tiles
        let mx = event_pump.mouse_state().x() as u32;
        let my = event_pump.mouse_state().y() as u32;
        let md = event_pump.mouse_state().is_mouse_button_pressed(sdl2::mouse::MouseButton::Left);

        if state.mouse_pressed_previous && !md { state.mouse_pressed_previous = false; }

        let mut mouse_over_coord: Option<String> = None;

        for y in 0..8_u8 {
            for x in 0..8_u8 {
                let square = state[(x, y)];
                let top_left_onscreen = (x as u32 * SQUARE_W, (7 - y) as u32 * SQUARE_W);

                let mouse_hit = mx >= top_left_onscreen.0 && mx < top_left_onscreen.0 + SQUARE_W &&
                                my >= top_left_onscreen.1 && my < top_left_onscreen.1 + SQUARE_W;

                if mouse_hit { mouse_over_coord = Some(square.coord()) };

                if mouse_hit && !state.mouse_pressed_previous && md {
                    if state.selected_square.is_none() {
                        if square.content.is_some() && square.content.unwrap().colour == state.turn {
                            state.selected_square = Some((x, y));
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
                canvas.fill_rect(screen_rect)?;

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

                    canvas.copy(texture, None, screen_rect)?;
                }
            }
        }

        if let Some(text) = mouse_over_coord {
            let drawn_text = if state.selected_square.is_none() { text } else {
                let square = state[state.selected_square.unwrap()];
                format!("{} -> {}", square.coord(), text)
            };
            draw_text(&drawn_text, &mut canvas, &texture_creator, &font, BOARD_EDGE + MARGIN, SCREEN_H as i32 - 2*MARGIN)?;
        }

        draw_text(&format!("{} to play", if state.turn == ChessColour::White { "white" } else { "black" }), &mut canvas, &texture_creator, &font, BOARD_EDGE + MARGIN, MARGIN)?;

        if let Some(coord) = state.selected_square {
            canvas.set_draw_color(Color::RGBA(50, 200, 20, 50));
            
            let valid_moves = get_moves(&state, coord);
            for y in 0..8_u32 {
                for x in 0..8_u32 {
                    if valid_moves.contains(&(x as u8, y as u8)) {
                        let cx = (x * SQUARE_W + SQUARE_W / 2) as i32;
                        let cy = ((7 - y) * SQUARE_W + SQUARE_W / 2) as i32;
                        canvas.fill_rect(Rect::from_center((cx, cy), SQUARE_W / 3, SQUARE_W / 3))?;
                    }
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

    Ok(())
}

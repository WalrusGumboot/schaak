const SQUARE_W: u32 = 55;
const BOARD_EDGE: i32 = 8 * SQUARE_W as i32;
const MARGIN: i32 = 16; // obv only makes sense as unsigned, but this makes addition nicer
const SCREEN_W: u32 = BOARD_EDGE as u32 + 400;
const SCREEN_H: u32 = BOARD_EDGE as u32;

use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::{self, Font};
use sdl2::video::{Window, WindowContext};
use std::time::Duration;

mod piece;
use piece::{PieceKind::*, *};

mod chess_move;
use chess_move::*;

mod state;
use state::*;

mod square;

mod player;
use player::*;

fn draw_text(
    text: &str,
    c: &mut Canvas<Window>,
    tc: &TextureCreator<WindowContext>,
    font: &Font,
    x: i32,
    y: i32,
) -> Result<(), String> {
    let turn_text = font.render(text);
    let text_surface = turn_text.solid(Color::WHITE).unwrap();
    let text_texture = text_surface.as_texture(tc).unwrap();

    c.copy(
        &text_texture,
        None,
        Rect::new(x, y, text_surface.width(), text_surface.height()),
    )
}

fn main() -> Result<(), String> {
    let mut state = State::new();

    let sdl_context = sdl2::init().unwrap();
    let _image_context = image::init(InitFlag::PNG).unwrap(); // has to be let-binding to ensure drop at the end of the program
    let font_context = ttf::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let font = font_context
        .load_font("assets/fonts/input.ttf", 16)
        .unwrap();

    let window = video_subsystem
        .window("schaak", SCREEN_W, SCREEN_H)
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();
    let tex_wp = texture_creator
        .load_texture("assets/textures/wp.png")
        .unwrap();
    let tex_wr = texture_creator
        .load_texture("assets/textures/wr.png")
        .unwrap();
    let tex_wn = texture_creator
        .load_texture("assets/textures/wn.png")
        .unwrap();
    let tex_wb = texture_creator
        .load_texture("assets/textures/wb.png")
        .unwrap();
    let tex_wq = texture_creator
        .load_texture("assets/textures/wq.png")
        .unwrap();
    let tex_wk = texture_creator
        .load_texture("assets/textures/wk.png")
        .unwrap();
    let tex_bp = texture_creator
        .load_texture("assets/textures/bp.png")
        .unwrap();
    let tex_br = texture_creator
        .load_texture("assets/textures/br.png")
        .unwrap();
    let tex_bn = texture_creator
        .load_texture("assets/textures/bn.png")
        .unwrap();
    let tex_bb = texture_creator
        .load_texture("assets/textures/bb.png")
        .unwrap();
    let tex_bq = texture_creator
        .load_texture("assets/textures/bq.png")
        .unwrap();
    let tex_bk = texture_creator
        .load_texture("assets/textures/bk.png")
        .unwrap();

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
        let md = event_pump
            .mouse_state()
            .is_mouse_button_pressed(sdl2::mouse::MouseButton::Left);

        if state.mouse_pressed_previous && !md {
            state.mouse_pressed_previous = false;
        }

        let mut mouse_over_coord: Option<String> = None;

        for y in 0..8_u8 {
            for x in 0..8_u8 {
                let square = state[(x, y)];
                let top_left_onscreen = (x as u32 * SQUARE_W, (7 - y) as u32 * SQUARE_W);

                let mouse_hit = mx >= top_left_onscreen.0
                    && mx < top_left_onscreen.0 + SQUARE_W
                    && my >= top_left_onscreen.1
                    && my < top_left_onscreen.1 + SQUARE_W;

                if mouse_hit {
                    mouse_over_coord = Some(square.coord())
                };

                // we only want interaction when the game is running;
                // this mouse check is the only place where input happens.

                if state.game_running {
                    if mouse_hit && !state.mouse_pressed_previous && md {
                        if state.selected_square.is_none() {
                            if square.content.is_some()
                                && square.content.unwrap().colour == state.turn
                            {
                                state.selected_square = Some((x, y));
                            }
                        } else {
                            if state.attempt_move((x, y)) {
                                state.turn = state.turn.flip();
                                // every piece that is of the colour whose turn it currently is can now be "de-en passanted"; it is no longer the current turn
                                let aux_squares = state.squares.clone();
                                for (idx, s) in aux_squares.into_iter().enumerate() {
                                    if let Some(p) = s.content {
                                        if p.colour == state.turn {
                                            state.squares[idx].content = Some(Piece {
                                                en_passanteable: false,
                                                ..p
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let screen_rect = Rect::new(
                    top_left_onscreen.0 as i32,
                    top_left_onscreen.1 as i32,
                    SQUARE_W,
                    SQUARE_W,
                );

                canvas.set_draw_color(square.colour(mouse_hit));
                if state[(x, y)].content.is_some() && state[(x, y)].content.unwrap().en_passanteable
                {
                    canvas.set_draw_color(Color::RGB(140, 100, 250));
                }
                if state.selected_square.is_some() && state.selected_square.unwrap() == (x, y) {
                    canvas.set_draw_color(Color::RGB(240, 200, 210));
                }
                canvas.fill_rect(screen_rect)?;

                if let Some(piece) = square.content {
                    let texture = match piece {
                        Piece {
                            kind: PieceKind::Pawn,
                            colour: ChessColour::White,
                            ..
                        } => &tex_wp,
                        Piece {
                            kind: PieceKind::Rook,
                            colour: ChessColour::White,
                            ..
                        } => &tex_wr,
                        Piece {
                            kind: PieceKind::Knight,
                            colour: ChessColour::White,
                            ..
                        } => &tex_wn,
                        Piece {
                            kind: PieceKind::Bishop,
                            colour: ChessColour::White,
                            ..
                        } => &tex_wb,
                        Piece {
                            kind: PieceKind::Queen,
                            colour: ChessColour::White,
                            ..
                        } => &tex_wq,
                        Piece {
                            kind: PieceKind::King,
                            colour: ChessColour::White,
                            ..
                        } => &tex_wk,

                        Piece {
                            kind: PieceKind::Pawn,
                            colour: ChessColour::Black,
                            ..
                        } => &tex_bp,
                        Piece {
                            kind: PieceKind::Rook,
                            colour: ChessColour::Black,
                            ..
                        } => &tex_br,
                        Piece {
                            kind: PieceKind::Knight,
                            colour: ChessColour::Black,
                            ..
                        } => &tex_bn,
                        Piece {
                            kind: PieceKind::Bishop,
                            colour: ChessColour::Black,
                            ..
                        } => &tex_bb,
                        Piece {
                            kind: PieceKind::Queen,
                            colour: ChessColour::Black,
                            ..
                        } => &tex_bq,
                        Piece {
                            kind: PieceKind::King,
                            colour: ChessColour::Black,
                            ..
                        } => &tex_bk,
                    };

                    canvas.copy(texture, None, screen_rect)?;
                }
            }
        }

        if let Some(text) = mouse_over_coord {
            let drawn_text = if state.selected_square.is_none() {
                text
            } else {
                let square = state[state.selected_square.unwrap()];
                format!("{} -> {}", square.coord(), text)
            };
            draw_text(
                &drawn_text,
                &mut canvas,
                &texture_creator,
                &font,
                BOARD_EDGE + MARGIN,
                SCREEN_H as i32 - 2 * MARGIN,
            )?;
        }

        if state.game_running {
            draw_text(
                &format!(
                    "{} to play",
                    if state.turn == ChessColour::White {
                        "white"
                    } else {
                        "black"
                    }
                ),
                &mut canvas,
                &texture_creator,
                &font,
                BOARD_EDGE + MARGIN,
                MARGIN,
            )?;
        } else {
            draw_text(
                "checkmate!",
                &mut canvas,
                &texture_creator,
                &font,
                BOARD_EDGE + MARGIN,
                MARGIN,
            )?;
        }

        if let Some(coord) = state.selected_square {
            canvas.set_draw_color(Color::RGBA(50, 200, 20, 50));

            let valid_moves = state.get_moves(coord, true);
            for y in 0..8_u32 {
                for x in 0..8_u32 {
                    if valid_moves.contains(&ChessMove::dummy((x as u8, y as u8))) {
                        let cx = (x * SQUARE_W + SQUARE_W / 2) as i32;
                        let cy = ((7 - y) * SQUARE_W + SQUARE_W / 2) as i32;
                        canvas.fill_rect(Rect::from_center(
                            (cx, cy),
                            SQUARE_W / 3,
                            SQUARE_W / 3,
                        ))?;
                    }
                }
            }
        }

        // checkmate test
        for c in &[ChessColour::White, ChessColour::Black] {
            if state.is_in_check(*c) {
                // test every piece's every move and check if the king is still in check
                let no_unchecking_moves = state
                    .squares
                    .into_iter()
                    .filter(|s| {
                        if let Some(p) = s.content {
                            p.colour == *c
                        } else {
                            false
                        }
                    })
                    .map(|s| (s.coords, state.get_moves(s.coords, true)))
                    .map(|(square, moves)| {
                        // we will test if making the move still leaves the king in check
                        moves.into_iter().all(|m| {
                            let mut test_board = state.clone();
                            test_board.make_move(square, m);

                            test_board.is_in_check(*c)
                        })
                    })
                    .all(|b| b);

                if no_unchecking_moves {
                    state.game_running = false;
                }
            }
        }

        draw_text(
            &format!(
                "promotion: {} {} {} {}",
                if state.next_promotor == Queen {
                    "[Q]"
                } else {
                    " Q "
                },
                if state.next_promotor == Rook {
                    "[R]"
                } else {
                    " R "
                },
                if state.next_promotor == Bishop {
                    "[B]"
                } else {
                    " B "
                },
                if state.next_promotor == Knight {
                    "[N]"
                } else {
                    " N "
                },
            ),
            &mut canvas,
            &texture_creator,
            &font,
            BOARD_EDGE + MARGIN,
            3 * MARGIN,
        )?;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => {
                    state.next_promotor = Queen;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    state.next_promotor = Rook;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::B),
                    ..
                } => {
                    state.next_promotor = Bishop;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::N),
                    ..
                } => {
                    state.next_promotor = Knight;
                }

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

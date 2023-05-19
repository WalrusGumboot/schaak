use rand::prelude::*;
use rand::rngs::ThreadRng;

use crate::state::State;
use crate::{chess_move::ChessMove, piece::ChessColour};

use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Clone)]
pub struct MoveInfo {
    pub coord: (u8, u8),
    pub move_data: ChessMove,
}

unsafe impl Send for MoveInfo {}

pub trait Player {
    fn send_move(&mut self) -> Result<(), mpsc::SendError<MoveInfo>>;
    fn update_internal_state(&mut self, new_move: MoveInfo);
    fn tick(&mut self);
}

pub struct HumanPlayer {
    internal_state: State,
    selected_move: Option<MoveInfo>,
    tx_to_main: Sender<MoveInfo>,
    rx_from_main: Receiver<MoveInfo>,
    colour: ChessColour,
}

impl HumanPlayer {
    pub fn new(
        state_to_clone: &State,
        rx_from_main: Receiver<MoveInfo>,
        colour: ChessColour,
    ) -> (Self, Receiver<MoveInfo>) {
        let (tx_to_main, rx) = mpsc::channel();

        (
            HumanPlayer {
                internal_state: state_to_clone.clone(),
                selected_move: None,
                tx_to_main,
                rx_from_main,
                colour,
            },
            rx,
        )
    }
}

impl Player for HumanPlayer {
    fn send_move(&mut self) -> Result<(), mpsc::SendError<MoveInfo>> {
        assert!(
            self.selected_move.is_some(),
            "Cannot send a move; no move selected."
        );

        let mi = self.selected_move.clone().unwrap();

        self.tx_to_main.send(mi)?;

        self.selected_move = None;

        Ok(())
    }

    fn update_internal_state(&mut self, new_move: MoveInfo) {
        self.internal_state
            .make_move(new_move.coord, new_move.move_data);
    }

    fn tick(&mut self) {
        if let Ok(new_move) = self.rx_from_main.try_recv() {
            self.update_internal_state(new_move);
        }

        if self.selected_move.is_some() {
            self.send_move().unwrap_or_else(|err| panic!("err: {err}"));
        }
    }
}

unsafe impl Send for HumanPlayer {}

pub struct RandomPlayer {
    internal_state: State,
    selected_move: Option<MoveInfo>,
    tx_to_main: Sender<MoveInfo>,
    rx_from_main: Receiver<MoveInfo>,
    colour: ChessColour,
    rng: ThreadRng,
}

impl RandomPlayer {
    pub fn new(
        state_to_clone: &State,
        rx_from_main: Receiver<MoveInfo>,
        colour: ChessColour,
    ) -> (Self, Receiver<MoveInfo>) {
        let (tx_to_main, rx) = mpsc::channel();
        let thread_rng = thread_rng();
        (
            RandomPlayer {
                internal_state: state_to_clone.clone(),
                selected_move: None,
                tx_to_main,
                rx_from_main,
                colour,
                rng: thread_rng,
            },
            rx,
        )
    }
}

impl Player for RandomPlayer {
    fn send_move(&mut self) -> Result<(), mpsc::SendError<MoveInfo>> {
        assert!(
            self.selected_move.is_some(),
            "Cannot send a move; no move selected."
        );

        let mi = self.selected_move.clone().unwrap();

        self.tx_to_main.send(mi)?;

        self.selected_move = None;

        Ok(())
    }

    fn update_internal_state(&mut self, new_move: MoveInfo) {
        self.internal_state
            .make_move(new_move.coord, new_move.move_data);
    }

    fn tick(&mut self) {
        if let Ok(new_move) = self.rx_from_main.try_recv() {
            self.update_internal_state(new_move);
        }

        if self.selected_move.is_some() {
            self.send_move().unwrap_or_else(|err| panic!("err: {err}"));
        }

        if self.selected_move.is_none() {
            // we'll pick a random move here
            let mut all_moves = self
                .internal_state
                .squares
                .iter()
                .filter(|s| s.content.is_some())
                .map(|s| (s.coords, s.content.unwrap()))
                .filter(|(_, p)| p.colour == self.colour)
                .map(|(c, _)| (c, self.internal_state.get_moves(c, true)))
                .map(|(c, m)| {
                    m.iter()
                        .cloned()
                        .map(|c_move| (c, c_move))
                        .collect::<Vec<_>>()
                })
                .flatten()
                .map(|(c, m)| MoveInfo {
                    coord: c,
                    move_data: m.clone(),
                })
                .collect::<Vec<_>>();

            all_moves.shuffle(&mut self.rng);

            self.selected_move = Some(all_moves[1].clone())
        }
    }
}

unsafe impl Send for RandomPlayer {}

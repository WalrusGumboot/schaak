use crate::chess_move::ChessMove;
use crate::state::State;

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
}

impl HumanPlayer {
    pub fn new(
        state_to_clone: &State,
        rx_from_main: Receiver<MoveInfo>,
    ) -> (Self, Receiver<MoveInfo>) {
        let (tx_to_main, rx) = mpsc::channel();

        (
            HumanPlayer {
                internal_state: state_to_clone.clone(),
                selected_move: None,
                tx_to_main,
                rx_from_main,
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

        self.tx_to_main.send(mi.clone())?;

        self.internal_state.make_move(mi.coord, mi.move_data);

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
    }
}

unsafe impl Send for HumanPlayer {}

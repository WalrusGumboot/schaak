use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::state::State;
use crate::{chess_move::MoveInfo, piece::ChessColour};

use std::sync::mpsc::{self, Receiver, Sender};

pub trait Player: Send + Sized {
    // takes the receiver from main, which sends moves when they arrive from both this and the other player
    // returns the struct and the receiving end for moves transmitted from the struct to main
    fn new(
        rx_from_main: Receiver<MoveInfo>,
        state_to_clone: &State,
        colour: ChessColour,
    ) -> (Self, Receiver<MoveInfo>);

    // updates coming in from main thread
    fn receive_move_from_main(&mut self) -> Result<MoveInfo, mpsc::TryRecvError>;
    // includes player's own moves! they are communicated back by main thread
    fn apply_move(&mut self, mi: MoveInfo);

    // move generation and sending to main thread
    fn return_new_move(&self) -> Option<MoveInfo>;
    fn ponder_new_move(&mut self);
    fn send_move_to_main(&mut self) -> Result<(), mpsc::SendError<MoveInfo>>;

    // general loop
    fn tick(&mut self) {
        // potential overload
        self.specific_tick();

        if self.return_new_move().is_some() {
            println!("└─ new move is ready!");
            self.send_move_to_main()
                .expect("Could not send new move to main");
            println!("└─ sent new move to main thread.");
        }

        if let Ok(new_move_to_be_applied) = self.receive_move_from_main() {
            println!(
                "├─ new move received, from coord {:?}",
                new_move_to_be_applied.coord
            );
            self.apply_move(new_move_to_be_applied);
        } else {
            println!("├─ no new move received. ");
            self.ponder_new_move();
            println!("└─ started pondering new move.");
        }
    }

    fn specific_tick(&mut self) {}
}

pub struct RandomPlayer {
    tx_to_main: Sender<MoveInfo>,
    rx_from_main: Receiver<MoveInfo>,

    // constructor takes a reference but that gets cloned over
    // could in theory be useful for e.g. switching to Stockfish mid-game
    internal_state: State,

    // proper move
    move_info: Option<MoveInfo>,

    // colour which this player adopts
    colour: ChessColour,

    // exclusive to RandomPlayer
    rng: SmallRng,
}

impl Player for RandomPlayer {
    fn new(
        rx_from_main: Receiver<MoveInfo>,
        state_to_clone: &State,
        colour: ChessColour,
    ) -> (Self, Receiver<MoveInfo>) {
        let (own_tx, own_rx) = mpsc::channel();

        (
            RandomPlayer {
                rx_from_main,
                tx_to_main: own_tx,
                internal_state: state_to_clone.clone(),
                move_info: None,
                rng: SmallRng::from_entropy(),
                colour,
            },
            own_rx,
        )
    }

    fn apply_move(&mut self, mi: MoveInfo) {
        self.internal_state.make_move(mi.coord, mi.move_data);
    }

    fn receive_move_from_main(&mut self) -> Result<MoveInfo, mpsc::TryRecvError> {
        self.rx_from_main.try_recv()
    }

    fn return_new_move(&self) -> Option<MoveInfo> {
        self.move_info.clone()
    }

    fn send_move_to_main(&mut self) -> Result<(), mpsc::SendError<MoveInfo>> {
        self.tx_to_main.send(self.move_info.clone().unwrap())?;
        self.move_info = None;

        Ok(())
    }

    fn ponder_new_move(&mut self) {
        let mut currently_available_moves =
            self.internal_state.get_all_moves_for_colour(self.colour);
        currently_available_moves.shuffle(&mut self.rng);

        let selected_move = currently_available_moves[0].clone();

        self.move_info = Some(MoveInfo {
            coord: selected_move.0,
            move_data: selected_move.1,
        });
    }

    fn specific_tick(&mut self) {
        println!("tick from {:?} player", self.colour);
    }
}

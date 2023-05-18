use crate::chess_move::ChessMove;
use crate::square::Square;
use crate::state::State;

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

pub enum ChessMessage {}

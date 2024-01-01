pub mod datacollector;
pub mod runner;

use game::{
    board::{Board, Status},
    movegen::Move,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub boards: Vec<Game>,
}
#[derive(Serialize, Deserialize)]
pub struct Game {
    pub start: Board,
    pub moves: Vec<Move>,
    pub status: Status,
}
// Openings, contains a vector of opening board states.
#[derive(Clone)]
pub struct Openings {
    pub openings: Vec<Board>,
}

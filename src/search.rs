use std::sync::{Arc, Mutex};

use crate::{
    board::Board,
    move_app::make_move,
    movegen::{generate_moves, Move},
    GoInfo, Shared,
};

pub struct Search {
    pub shared: Arc<Mutex<Shared>>,
    pub board: Board,
}

impl Search {
    /// do the things like clearing the hash, resetting history, etc
    pub fn setup_newgame(&mut self) {}
    /// initialize the board state using stuff
    pub fn set_position(&mut self, input: String) {
        let is_startpos = input.contains("startpos");
        if is_startpos {
            self.board = Board::new("x5o/7/7/7/7/7/o5x x 0 1".to_string());
        } else {
            let end_index = if input.contains("moves") {
                input.find("moves").unwrap()
            } else {
                input.len()
            };
            let start_index = input.find("fen").unwrap() + 4;
            self.board = Board::new(input[start_index..end_index].to_string());
        }
        if input.contains("moves") {
            let begin_index = input.find("moves").unwrap() + 5;
            let moves: Vec<&str> = input[begin_index..input.len()].split(' ').collect();
            for mov in moves {
                let other_pieces = self.board.other_pieces();
                make_move(&mut self.board, &Move::from_str(mov, other_pieces));
            }
        }
    }

    /// find the best move for a position
    pub fn find_best_move(&mut self, info: &GoInfo) {
        println!("bestmove {}", generate_moves(&self.board)[0]);
    }
}

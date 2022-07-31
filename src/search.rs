use std::sync::{Arc, Mutex};

use crate::{
    board::{Board, Side, Status},
    move_app::{make_move, unmake_move},
    movegen::{generate_moves, singles, Move},
    GoInfo, Shared,
};

pub struct Search {
    pub shared: Arc<Mutex<Shared>>,
    pub board: Board,
    pub my_side: Side,
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

        self.my_side = self.board.side_to_move;
    }

    /// find the best move for a position
    pub fn find_best_move(&mut self, _info: &GoInfo) {
        let mut bestmove = Move {
            null: false,
            from: 0,
            to: 0,
            capture_square: 0,
        };
        self.negamax(i32::MIN, i32::MAX, 3, &mut bestmove);
        println!("bestmove {}", bestmove);
    }

    /// negamax
    pub fn negamax(&mut self, mut alpha: i32, beta: i32, depth: u8, out: &mut Move) -> i32 {
        if depth == 0 || self.board.game_over() {
            return match self.board.status() {
                Status::Draw => 0,
                Status::Winner => 1000,
                Status::Loser => -1000,
                Status::Ongoing => self.eval() * self.board.side_to_move.toi32(),
            };
        }

        let mut score = i32::MIN;
        let mut best_move = Move {
            null: false,
            from: 0,
            to: 0,
            capture_square: 0,
        };
        for mov in &generate_moves(&self.board) {
            let delta = make_move(&mut self.board, mov);
            let value = -self.negamax(-beta, -alpha, depth - 1, out);
            unmake_move(&mut self.board, mov, delta);
            if value > score {
                best_move = *mov;
            }
            score = score.max(value);
            // alpha = alpha.max(score);
            // if alpha >= beta {
            //     break; // cutoff
            // }
        }
        *out = best_move;
        score
    }

    fn eval(&self) -> i32 {
        self.board.boards[self.board.side_to_move as usize].count_ones() as i32
            - self.board.boards[1 - self.board.side_to_move as usize].count_ones() as i32
            + singles(self.board.boards[self.board.side_to_move as usize]).count_ones() as i32
    }
}

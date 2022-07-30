use std::sync::{Arc, Mutex};

use crate::{
    board::{Board, Side, Status},
    move_app::make_move,
    movegen::{generate_moves, Move},
    GoInfo, Shared,
};

pub struct Search {
    pub shared: Arc<Mutex<Shared>>,
    pub board: Board,
    pub my_side: Side,
}

#[derive(Debug, Clone, Copy)]
pub struct Eval {
    pub score: i32,
    pub mov: Option<Move>,
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
        let bestmove = self.negamax(i32::MIN, i32::MAX, 6);
        println!("bestmove {}", bestmove.mov.unwrap());
    }

    /// negamax
    pub fn negamax(&mut self, mut alpha: i32, beta: i32, depth: u8) -> Eval {
        if depth == 0 || self.board.game_over() {
            return match self.board.status() {
                Status::Draw => Eval {
                    score: 0,
                    mov: None,
                },
                Status::Winner(side) => match side {
                    Side::Black => Eval {
                        score: -1000,
                        mov: None,
                    },
                    Side::White => Eval {
                        score: 1000,
                        mov: None,
                    },
                },
                Status::Ongoing => Eval {
                    score: self.eval() * self.board.side_to_move.toi32(),
                    mov: None,
                },
            };
        }

        let mut out = Eval {
            score: i32::MIN,
            mov: None,
        };
        for mov in &generate_moves(&self.board) {
            let eval = self.negamax(-alpha, -beta, depth - 1);

            if -eval.score > out.score {
                out.score = -eval.score;
                out.mov = Some(*mov);
            }
            alpha = alpha.max(out.score);
            if alpha >= beta {
                break; // cutoff
            }
        }
        out
    }

    fn eval(&self) -> i32 {
        self.board.boards[self.board.side_to_move as usize].count_ones() as i32
            - self.board.boards[1 - self.board.side_to_move as usize].count_ones() as i32
    }
}

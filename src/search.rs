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
    }

    /// find the best move for a position
    pub fn find_best_move(&mut self, _info: &GoInfo) {
        let bestmove = self.minimax(self.board.side_to_move.to_bool(), i32::MIN, i32::MAX, 4);
        println!("bestmove {}", bestmove.mov.unwrap());
    }

    /// minimax
    pub fn minimax(&mut self, maximizing: bool, mut alpha: i32, mut beta: i32, depth: u8) -> Eval {
        /*
        function alphabeta(node, depth, α, β, maximizingPlayer) is
            if depth = 0 or node is a terminal node then
                return the heuristic value of node
            if maximizingPlayer then
                value := −∞
                for each child of node do
                    value := max(value, alphabeta(child, depth − 1, α, β, FALSE))
                    α := max(α, value)
                    if value ≥ β then
                        break (* β cutoff *)
                return value
            else
                value := +∞
                for each child of node do
                    value := min(value, alphabeta(child, depth − 1, α, β, TRUE))
                    β := min(β, value)
                    if value ≤ α then
                        break (* α cutoff *)
                return value
                    */

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
                    score: self.eval(),
                    mov: None,
                },
            };
        }

        if maximizing {
            let mut out = Eval {
                score: i32::MIN,
                mov: None,
            };
            for mov in &generate_moves(&self.board) {
                let eval = self.minimax(!maximizing, alpha, beta, depth - 1);

                if eval.score > out.score {
                    out.score = eval.score;
                    out.mov = Some(*mov);
                }
                alpha = alpha.max(out.score);
                if out.score >= beta {
                    break; // beta cutoff
                }
            }
            out
        } else {
            let mut out = Eval {
                score: i32::MAX,
                mov: None,
            };
            for mov in &generate_moves(&self.board) {
                let eval = self.minimax(!maximizing, alpha, beta, depth - 1);
                if eval.score < out.score {
                    out.score = eval.score;
                    out.mov = Some(*mov);
                }

                beta = beta.max(out.score);
                if out.score <= alpha {
                    break; // alpha cutoff
                }
            }
            out
        }
    }

    fn eval(&self) -> i32 {
        self.board.boards[Side::White as usize].count_ones() as i32
            - self.board.boards[Side::Black as usize].count_ones() as i32
    }
}

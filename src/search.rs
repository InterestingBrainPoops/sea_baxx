use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    board::{Board, Side, Status},
    move_app::{make_move, unmake_move},
    movegen::{generate_moves, Move},
    GoInfo, Shared,
};

pub struct Search {
    pub shared: Arc<Mutex<Shared>>,
    pub board: Board,
    pub my_side: Side,
}

pub struct Controller {
    pub end_time: Instant,
    pub max_depth: u8,
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
    pub fn find_best_move(&mut self, info: &GoInfo) {
        let my_time;
        let other_time;
        match self.my_side {
            Side::Black => {
                my_time = info.btime.unwrap();
                other_time = info.wtime.unwrap();
            }
            Side::White => {
                my_time = info.wtime.unwrap();
                other_time = info.btime.unwrap();
            }
        };
        let time_left = if other_time < my_time {
            (my_time - other_time).max(my_time - other_time + 30)
        } else {
            my_time / 10
        };
        let mut controller = Controller {
            end_time: Instant::now() + Duration::from_millis(time_left.into()),
            max_depth: 1,
        };
        let mut bestmove = Move {
            null: false,
            from: 0,
            to: 0,
            capture_square: 0,
        };
        let mut root = Node::new(&self.board);
        while Instant::now() < controller.end_time {
            self.iterate(&mut root);
        }

        println!("bestmove {}", bestmove);
    }
    fn iterate(&self, node: &mut Node) {
        let mut leaf = node;
        let mut path = vec![leaf];
        while !leaf.children.iter().any(|x| x.num_simulations == 0) {
            leaf = leaf.best_ucb();
        }
    }
}

#[derive(Debug, Clone)]
struct Node {
    children: Vec<Node>,
    board: Board,
    num_simulations: u32,
    cumulative_score: f64,
}

impl Node {
    fn new(board: &Board) -> Node {
        todo!();
    }
    fn best_ucb(&mut self) -> &mut Node {
        &mut self.children[0]
    }
    fn rollout(&self) -> f64 {
        todo!();
    }
    fn expand(&mut self) {}
}

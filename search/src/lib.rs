mod movepicker;
mod table;

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::movepicker::MovePicker;
use crate::table::{Entry, NodeType, Table};
use eval::Eval;
use game::{
    board::{Board, Side, Status},
    move_app::{make_move, unmake_move},
    movegen::{generate_moves, Move},
};
pub struct Shared {
    pub stop: bool,
}
pub struct GoInfo {
    pub wtime: Option<u32>,
    pub btime: Option<u32>,
    pub winc: Option<u32>,
    pub binc: Option<u32>,
    pub moves_to_go: Option<u32>,
    pub depth: Option<u32>,
    pub nodes: Option<u32>,
    pub mate: Option<u32>,
    pub movetime: Option<u32>,
    pub infinite: bool,
}
macro_rules! find_arg {
    ($split : ident , $x: expr, $y : ty) => {
        if $split.contains(&$x) {
            let x = $split.iter().position(|&r| r == $x).unwrap() + 1;
            Some($split[x].parse::<$y>().unwrap())
        } else {
            None
        }
    };
}

impl GoInfo {
    pub fn new(input: String) -> Self {
        let split: Vec<&str> = input.split(' ').collect();
        let out = Self {
            wtime: find_arg!(split, "wtime", u32),
            btime: find_arg!(split, "btime", u32),
            winc: find_arg!(split, "winc", u32),
            binc: find_arg!(split, "binc", u32),
            moves_to_go: find_arg!(split, "movestogo", u32),
            depth: find_arg!(split, "depth", u32),
            nodes: find_arg!(split, "nodes", u32),
            mate: find_arg!(split, "mate", u32),
            movetime: find_arg!(split, "movetime", u32),
            infinite: {
                if split.contains(&"infinite") {
                    true
                } else {
                    false
                }
            },
        };
        out
    }
}

pub struct Search {
    nodes: u64,
    shared: Arc<Mutex<Shared>>,
    table: Table,
    board: Board,
    my_side: Side,
    stack_storage: Vec<SearchData>,
    eval: Eval,
}

pub struct SearchData {
    killer_move: Option<Move>,
    pv_move: Option<Move>,
}
pub struct Controller {
    pub end_time: Instant,
    pub max_depth: u8,
}

impl Search {
    pub fn new(shared: Arc<Mutex<Shared>>) -> Self {
        Search {
            stack_storage: vec![],
            nodes: 0,
            table: Table::new(2_000_000),
            shared: shared,
            board: Board::new("x5o/7/7/7/7/7/o5x x 0 1".to_string()),
            my_side: Side::Black,
            eval: Eval::new(),
        }
    }
    /// do the things like clearing the hash, resetting history, etc
    pub fn setup_newgame(&mut self) {
        self.table.reset();
        self.stack_storage = vec![];
    }
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
        let mut controller;
        if info.infinite {
            controller = Controller {
                end_time: Instant::now() + Duration::from_secs(1000),
                max_depth: 1,
            }
        } else {
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
            controller = Controller {
                end_time: Instant::now() + Duration::from_millis(time_left.into()),
                max_depth: 1,
            };
        }
        let t0 = Instant::now();
        let mut bestmove = Move {
            null: false,
            from: 0,
            to: 0,
            capture_square: 0,
        };
        self.stack_storage.push(SearchData {
            killer_move: None,
            pv_move: None,
        });
        let max_depth = if info.infinite { 255 } else { 50 };
        for depth in 1..max_depth {
            self.stack_storage.insert(
                0,
                SearchData {
                    killer_move: None,
                    pv_move: None,
                },
            );
            controller.max_depth = depth;
            self.nodes = 0;
            let score = self.negamax(&controller, -100_000, 100_000, depth);
            let t1 = Instant::now();
            println!(
                "info depth {depth} score {score}, nps {}, nodes {}, time {}",
                (self.nodes as f64 / (t1 - t0).as_secs_f64()) as u64,
                self.nodes,
                (t1 - t0).as_millis()
            );
            if t1 <= controller.end_time || self.shared.lock().unwrap().stop {
                bestmove = self.stack_storage[depth as usize].pv_move.unwrap();
            } else {
                break;
            }
        }

        println!("bestmove {}", bestmove);
    }

    /// negamax
    pub fn negamax(
        &mut self,
        controller: &Controller,
        mut alpha: i32,
        beta: i32,
        depth: u8,
    ) -> i32 {
        if Instant::now() > controller.end_time || self.shared.lock().unwrap().stop {
            return 0;
        }

        if depth == 0 || self.board.game_over() {
            return match self.board.status() {
                Status::Draw => 0,
                Status::Winner => 1000,
                Status::Loser => -1000,
                Status::Ongoing => self.eval.evaluate(&self.board),
            };
        }

        // probe tt
        let original_alpha = alpha;

        let mut best_score = i32::MIN;
        let mut best_move = Move {
            null: false,
            from: 0,
            to: 0,
            capture_square: 0,
        };
        let moves = generate_moves(&self.board);
        let mut tt_move = None;
        if let Some(entry) = &self.table[&self.board] {
            if entry.hash == self.board.hash() {
                // tt move
                if moves.contains(&entry.hash_move) {
                    tt_move = Some(entry.hash_move);
                }
            }
        }
        let mut killer_move = None;
        // // killer move
        if let Some(killer_entry) = self.stack_storage[depth as usize].killer_move {
            let num = if let Some(tt_move) = tt_move {
                tt_move == killer_entry
            } else {
                false
            };
            if moves.contains(&killer_entry) && !num {
                killer_move = Some(killer_entry);
            }
        }

        let movepicker = MovePicker::new(moves, tt_move, killer_move);

        let new_moves = movepicker.sort();

        for mov in &new_moves {
            self.nodes += 1;
            let delta = make_move(&mut self.board, mov);
            let score = -self.negamax(controller, -beta, -alpha, depth - 1);
            unmake_move(&mut self.board, mov, delta);

            if score > best_score {
                best_score = score;

                best_move = *mov;
                self.stack_storage[depth as usize].pv_move = Some(*mov);
                if score > alpha {
                    alpha = score;
                }
            }

            if alpha >= beta {
                self.stack_storage[depth as usize].killer_move = Some(*mov);
                break;
            }
        }

        let node_type = if best_score <= original_alpha {
            NodeType::Upper
        } else if best_score >= beta {
            NodeType::Lower
        } else {
            NodeType::Exact
        };

        self.table[&self.board] = Some(Entry::new(
            self.board.hash(),
            best_move,
            best_score,
            depth,
            node_type,
        ));

        best_score
    }
}

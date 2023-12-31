use std::{
    fmt::Alignment,
    slice::ChunksMut,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    board::{Board, Side, Status},
    move_app::{make_move, unmake_move},
    movegen::{generate_moves, singles, Move},
    movepicker::MovePicker,
    table::{Entry, NodeType, Table},
    GoInfo, Shared,
};

pub struct Search {
    pub nodes: u64,
    pub shared: Arc<Mutex<Shared>>,
    pub table: Table,
    pub board: Board,
    pub my_side: Side,
    pub stack_storage: Vec<SearchData>,
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
        for depth in 1..50 {
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
                (self.nodes as f64 / (t1 - t0).as_secs() as f64) as u64,
                self.nodes,
                (t1 - t0).as_millis()
            );
            if t1 <= controller.end_time {
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
        mut beta: i32,
        depth: u8,
    ) -> i32 {
        if Instant::now() > controller.end_time {
            return 0;
        }

        if depth == 0 || self.board.game_over() {
            return match self.board.status() {
                Status::Draw => 0,
                Status::Winner => 1000,
                Status::Loser => -1000,
                Status::Ongoing => self.eval(),
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
        let mut moves = generate_moves(&self.board);
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
    fn eval(&self) -> i32 {
        let us = self.board.boards[self.board.side_to_move as usize];
        let them = self.board.boards[1 - self.board.side_to_move as usize];
        let piece_count = us.count_ones() as i32 - them.count_ones() as i32;
        let mobilty =
            (singles(us) & !us).count_ones() as i32 - (singles(them) & !them).count_ones() as i32;

        piece_count
    }
}

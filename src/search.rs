use std::{
    slice::ChunksMut,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    board::{Board, Side, Status},
    move_app::{make_move, unmake_move},
    movegen::{generate_moves, singles, Move},
    table::{Entry, NodeType, Table},
    GoInfo, Shared,
};

pub struct Search {
    pub nodes: u64,
    pub shared: Arc<Mutex<Shared>>,
    pub table: Table,
    pub board: Board,
    pub my_side: Side,
}

pub struct Controller {
    pub end_time: Instant,
    pub max_depth: u8,
}

impl Search {
    /// do the things like clearing the hash, resetting history, etc
    pub fn setup_newgame(&mut self) {
        self.table.reset();
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
        for depth in 1..10 {
            controller.max_depth = depth;
            let mut mov = Move {
                null: false,
                from: 0,
                to: 0,
                capture_square: 0,
            };
            self.nodes = 0;
            let score = self.negamax(&controller, -100_000, 100_000, depth, &mut mov);
            let t1 = Instant::now();
            println!(
                "info depth {depth} score {score}, nps {}, nodes {}, time {}",
                (self.nodes as f64 / (t1 - t0).as_secs() as f64) as u64,
                self.nodes,
                (t1 - t0).as_millis()
            );
            if t1 <= controller.end_time {
                bestmove = mov;
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
        out: &mut Move,
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

        let mut best_score = i32::MIN;
        let mut best_move = Move {
            null: false,
            from: 0,
            to: 0,
            capture_square: 0,
        };
        let mut moves = generate_moves(&self.board);
        if let Some(entry) = &self.table[&self.board] {
            if let Some(killer_move) = entry.killer_move {
                if moves.contains(&killer_move) {
                    let index = moves
                        .iter()
                        .enumerate()
                        .find(|(_, mov)| *mov == &killer_move)
                        .unwrap()
                        .0;
                    moves.swap(0, index)
                }
            }
            // hash move in move ordering
        }
        let mut node_type = NodeType::Full;
        for mov in &moves {
            self.nodes += 1;
            let delta = make_move(&mut self.board, mov);
            let score = -self.negamax(controller, -beta, -alpha, depth - 1, out);
            unmake_move(&mut self.board, mov, delta);

            if score > best_score {
                best_move = *mov;
                best_score = score;

                alpha = alpha.max(score);

                if alpha >= beta {
                    node_type = NodeType::Cutoff;
                    break;
                }
            }
        }
        *out = best_move;
        let entry = &mut self.table[&self.board];
        if let Some(x) = entry {
            if node_type == NodeType::Cutoff {
                x.killer_move = Some(best_move);
            }
            if x.depth < depth && node_type == NodeType::Full {
                x.score = best_score;
                x.hash_move = best_move;
                x.node_type = node_type;
                x.depth = depth;
            }
        } else {
            self.table[&self.board] = Some(Entry::new(best_move, best_score, depth, node_type));
        }
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

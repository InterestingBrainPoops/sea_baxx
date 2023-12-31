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
    search_info: SearchInfo,
    shared: Arc<Mutex<Shared>>,
    table: Table,
    board: Board,
    my_side: Side,
    stack_storage: Vec<SearchData>,
    eval: Eval,
}

struct SearchInfo {
    nodes: u64,
    tt_hits: u64,
    cutoffs: u64,
}

impl SearchInfo {
    fn new() -> SearchInfo {
        SearchInfo {
            nodes: 0,
            tt_hits: 0,
            cutoffs: 0,
        }
    }

    fn reset(&mut self) {
        self.nodes = 0;
        self.tt_hits = 0;
        self.cutoffs = 0;
    }
}
#[derive(Clone, Copy)]
pub struct SearchData {
    killer_move: Option<Move>,
    pv_move: Option<Move>,
}

pub enum EndCondition {
    Time(Instant),
    Nodes(u64),
    Depth(u8),
    Infinite,
}

impl EndCondition {
    pub fn met(&self, nodes: u64, depth: u8) -> bool {
        match self {
            EndCondition::Time(end_time) => Instant::now() >= *end_time, // did we hit the time condition?
            EndCondition::Nodes(node_count) => nodes >= *node_count, // did we hit the node condition?
            EndCondition::Depth(depth_to_reach) => depth == *depth_to_reach, // did we hit the depth condition?
            EndCondition::Infinite => false, // you can never meet the end condition for infinity >:)
        }
    }
}
impl Search {
    pub fn new(shared: Arc<Mutex<Shared>>) -> Self {
        Search {
            stack_storage: vec![
                SearchData {
                    killer_move: None,
                    pv_move: None
                };
                200
            ],
            search_info: SearchInfo::new(),
            table: Table::new(2_000_000),
            shared: shared,
            board: Board::new("x5o/7/7/7/7/7/o5x x 0 1".to_string()),
            my_side: Side::Black,
            eval: Eval::new(),
        }
    }
    /// Clear hash and reset PV on new game
    pub fn setup_newgame(&mut self) {
        self.table.reset();
        self.stack_storage.iter_mut().for_each(|x| {
            *x = SearchData {
                killer_move: None,
                pv_move: None,
            }
        });
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
        // find run mode amongst : {infinite, time, depth, nodes, movetime}
        let end_cond;
        if info.infinite {
            end_cond = EndCondition::Infinite;
        } else if let Some(nodes) = info.nodes {
            end_cond = EndCondition::Nodes(nodes as u64);
        } else if let Some(depth) = info.depth {
            end_cond = EndCondition::Depth(depth as u8);
        } else if let Some(movetime) = info.movetime {
            end_cond = EndCondition::Time(Instant::now() + Duration::from_millis(movetime.into()));
        } else if let (Some(btime), Some(wtime)) = (info.btime, info.wtime) {
            let (binc, winc) = if let (Some(binc), Some(winc)) = (info.binc, info.winc) {
                (binc, winc)
            } else {
                (0, 0)
            };
            let my_time;
            let other_time;
            match self.my_side {
                Side::Black => {
                    my_time = btime;
                    other_time = wtime;
                }
                Side::White => {
                    my_time = wtime;
                    other_time = btime;
                }
            };
            let time_left = if other_time < my_time {
                (((my_time - other_time) * 3) / 4) + my_time / 10
            } else {
                my_time / 10
            };
            end_cond = EndCondition::Time(Instant::now() + Duration::from_millis(time_left.into()));
        } else {
            panic!("No end condition findable!");
        }
        // store the start time of the search for nps calcs
        let t0 = Instant::now();
        // store the best move
        let mut bestmove = Move {
            null: false,
            from: 0,
            to: 0,
            capture_square: 0,
        };

        let max_depth = 200; // randomly chosen
        self.search_info.reset();

        for depth in 1..max_depth {
            let score = self.negamax(&end_cond, -100_000, 100_000, depth);
            let t1 = Instant::now();
            println!(
                "info depth {depth} score {score}, nodes {}, time {}, tthits {}, cutoffs {}, nps {}",
                self.search_info.nodes,
                (t1 - t0).as_millis(), 
                self.search_info.tt_hits,
                self.search_info.cutoffs,
                (self.search_info.nodes as f64 / (t1 - t0).as_secs_f64()) as u64,
            );
            if end_cond.met(self.search_info.nodes, depth) || self.shared.lock().unwrap().stop {
                self.shared.lock().unwrap().stop = false;
                break;
            } else {
                bestmove = self.stack_storage[depth as usize].pv_move.unwrap();
            }
        }

        println!("bestmove {}", bestmove);
    }

    /// negamax
    pub fn negamax(
        &mut self,
        end_condition: &EndCondition,
        mut alpha: i32,
        beta: i32,
        depth: u8,
    ) -> i32 {
        // only worry about nodes cause search depth isnt useful here
        if end_condition.met(self.search_info.nodes, 0) || self.shared.lock().unwrap().stop {
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
                    self.search_info.tt_hits += 1;
                    tt_move = Some(entry.hash_move);
                }
            }
        }

        // killer move
        let mut killer_move = None;
        if let Some(killer_entry) = self.stack_storage[depth as usize].killer_move {
            let num = if let Some(tt_move) = tt_move {
                tt_move == killer_entry
            } else {
                false
            };
            if !num && moves.contains(&killer_entry) {
                killer_move = Some(killer_entry);
            }
        }

        let movepicker = MovePicker::new(moves, tt_move, killer_move);

        let new_moves = movepicker.sort();

        for mov in &new_moves {
            self.search_info.nodes += 1;
            let delta = make_move(&mut self.board, mov);
            let score = -self.negamax(end_condition, -beta, -alpha, depth - 1);
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
                // update killer if quiet move and single
                // if mov.capture_square == 0 {
                    self.stack_storage[depth as usize].killer_move = Some(*mov);
                // }
                self.search_info.cutoffs += 1;
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

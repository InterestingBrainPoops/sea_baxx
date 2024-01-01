use std::{
    cmp::Ordering,
    collections::HashMap,
    sync::{atomic::AtomicU64, mpsc::Sender, Arc, Mutex},
};

use game::{board::Status, move_app::make_move};
use rand::{seq::SliceRandom, thread_rng};
use search::{GoInfo, Search, Shared};

use crate::{Game, Openings};

pub struct Runner {
    searcher1: Search,
    searcher2: Search,
    openings: Openings,
}

impl Runner {
    pub fn new(openings: Openings) -> Runner {
        let val = Arc::new(Mutex::new(Shared { stop: false }));
        Runner {
            searcher1: Search::new(val.clone()),
            searcher2: Search::new(val.clone()),
            openings: openings,
        }
    }

    pub fn start(&mut self, _id: usize, num_games_left: Arc<Mutex<u32>>, send_pipe: Sender<Game>) {
        let mut current_opening = 0;
        let go_info = GoInfo {
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            moves_to_go: None,
            depth: Some(6),
            nodes: None,
            mate: None,
            movetime: None,
            infinite: false,
        };
        let mut rng = thread_rng();
        self.openings.openings.shuffle(&mut rng);
        while *num_games_left.lock().unwrap() != 0 {
            let mut moves = vec![];
            let mut game_state =
                self.openings.openings[current_opening % self.openings.openings.len()];
            let picker = (current_opening / self.openings.openings.len()) % 2 == 0;
            self.searcher1.setup_newgame();
            self.searcher2.setup_newgame();
            let mut x = 0;
            let mut fold_detector = HashMap::new();
            let mut draw = false;
            let mut discard = false;
            // println!("Game started {id}");

            while game_state.status() == Status::Ongoing {
                if x % 30 == 0 {
                    // println!("{id}: {x}");
                }

                if x == 700 {
                    discard = true;
                    // println!("Game discarded! id: {id}");
                    break;
                }
                x += 1;
                let (best_move, _) = if picker {
                    match game_state.side_to_move {
                        game::board::Side::Black => {
                            self.searcher1.set_position_direct(&game_state);
                            self.searcher1.find_best_move(false, &go_info)
                        }
                        game::board::Side::White => {
                            self.searcher2.set_position_direct(&game_state);
                            self.searcher2.find_best_move(false, &go_info)
                        }
                    }
                } else {
                    match game_state.side_to_move {
                        game::board::Side::White => {
                            self.searcher1.set_position_direct(&game_state);
                            self.searcher1.find_best_move(false, &go_info)
                        }
                        game::board::Side::Black => {
                            self.searcher2.set_position_direct(&game_state);
                            self.searcher2.find_best_move(false, &go_info)
                        }
                    }
                };

                make_move(&mut game_state, &best_move);
                if fold_detector.contains_key(&game_state.clone()) {
                    *fold_detector.get_mut(&game_state.clone()).unwrap() += 1;
                } else {
                    fold_detector.insert(game_state, 1);
                }

                moves.push(best_move);
                if let Some(val) = fold_detector.get(&game_state) {
                    if *val == 3 {
                        draw = true;
                        break;
                    }
                }
            }
            if discard {
                current_opening += 1;
                continue;
            }
            let status = if draw {
                Status::Draw
            } else {
                game_state.status()
            };
            if send_pipe
                .send(Game {
                    start: self.openings.openings[current_opening % self.openings.openings.len()],
                    moves,
                    status,
                })
                .is_err()
            {
                break;
            }

            *num_games_left.lock().unwrap() -= 1;
            // println!("game finished: {id}");
            current_opening += 1;
        }
    }
}

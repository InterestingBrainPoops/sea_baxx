mod board;
mod move_app;
mod movegen;
mod movepicker;
mod perft;
mod search;
mod table;

use std::{
    io::{self},
    sync::{mpsc::channel, Arc, Mutex},
    thread,
};
use text_io::read;

use crate::{
    board::{Board, Side},
    search::Search,
    table::Table,
};

fn get_input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");
    input
}

fn main() {
    // wait for uai

    loop {
        let input: String = read!();
        if input.trim_end() != "uai" {
            println!("Invalid protocol!");
        } else {
            break;
        }
    }
    // identify myself
    println!("id name SeaBaxx");
    println!("id author BrokenKeyboard");
    // send options

    // uciok
    println!("uaiok");
    // listen to option settings

    // // listen for isready
    // loop {
    //     let input: String = read!();
    //     if input != *"isready" {
    //         println!("Invalid input!");
    //     } else {
    //         break;
    //     }
    // }
    // setup my stuff
    let (send, recv) = channel::<SearchMessage>();
    let shared = Arc::new(Mutex::new(Shared { stop: false }));
    let shared_for_thread = Arc::clone(&shared);
    thread::spawn(move || {
        let mut search = Search {
            stack_storage: vec![],
            nodes: 0,
            table: Table::new(2_000_000),
            shared: Arc::clone(&shared_for_thread),
            board: Board::new("x5o/7/7/7/7/7/o5x x 0 1".to_string()),
            my_side: Side::Black,
        };
        while let Ok(message) = recv.recv() {
            match message {
                SearchMessage::NewGame => {
                    shared_for_thread.lock().expect("error").stop = false;
                    search.setup_newgame();
                }
                SearchMessage::Go(things) => {
                    search.find_best_move(&things);
                }
                SearchMessage::SetPosition(info) => {
                    search.set_position(info);
                }
                SearchMessage::Ready => {
                    println!("readyok");
                }
            }
        }
    });
    // send readyok
    // loop with a match for all the uai commands
    loop {
        let t = get_input();
        let input = t.trim();
        match input.split(' ').next().unwrap() {
            "uainewgame" => {
                send.send(SearchMessage::NewGame).unwrap();
            }
            "position" => {
                send.send(SearchMessage::SetPosition(String::from(
                    input.get(9..).unwrap(),
                )))
                .unwrap();
            }

            "go" => send
                .send(SearchMessage::Go(GoInfo::new(String::from(
                    input.get(2..).unwrap(),
                ))))
                .unwrap(),
            "stop" => {
                shared.lock().unwrap().stop = true;
            }
            "isready" => {
                send.send(SearchMessage::Ready).unwrap();
            }
            "ponderhit" => todo!(),
            "quit" => {
                break;
            }
            _ => {}
        }
    }
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

enum SearchMessage {
    NewGame,
    SetPosition(String),
    Go(GoInfo),
    Ready,
}

pub struct Shared {
    pub stop: bool,
}

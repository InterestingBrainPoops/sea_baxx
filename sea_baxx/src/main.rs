use search::Search;
use search::{GoInfo, Shared};

use std::{
    io::{self},
    sync::{mpsc::channel, Arc, Mutex},
    thread,
};
use text_io::read;

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
        let mut search = Search::new(Arc::clone(&shared_for_thread));
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

enum SearchMessage {
    NewGame,
    SetPosition(String),
    Go(GoInfo),
    Ready,
}

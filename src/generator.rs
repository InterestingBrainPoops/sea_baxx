use threadpool::ThreadPool;

use crate::board::{Board, Status};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    sync::{Arc, Barrier, Mutex},
    thread::Builder,
};

pub struct Generator {
    /// Config for the data generation cycle
    pub config: Config,

    /// starting book
    pub book: Book,
    /// completed games
    output: Arc<Mutex<Vec<Game>>>,
}

#[derive(Clone)]
pub struct Book {
    /// book
    book: Vec<Board>,
    /// book index
    index: u64,
}

impl Book {
    fn generate_job(&mut self, job_size: u64) -> Job {
        let mut starting_boards = vec![];
        for idx in self.index..job_size {
            starting_boards.push(self.book[idx as usize % self.book.len()]);
        }
        self.index += job_size;
        Job { starting_boards }
    }
}

pub struct Config {
    /// number of games needed
    num_games: u64,
    /// starting book path
    starting_book: String,
    /// output path to write to
    output_path: String,
    /// worker job size
    job_size: u64,
    /// number of paralell workers
    num_workers: u64,
}

pub struct Game {
    end_status: Status,
    /// stores positions starting from startpos all the way to the ending state (or current state if the game is incomplete)
    positions: Vec<Board>,
}

pub struct Job {
    starting_boards: Vec<Board>,
}

impl Generator {
    pub fn new(config: Config) -> Self {
        // load in the book
        let file = File::open("foo.txt").unwrap();
        let reader = BufReader::new(file);
        let mut book = vec![];
        for line in reader.lines() {
            book.push(Board::new(line.unwrap()))
        }
        Generator {
            config,
            book: Book { book, index: 0 },
            output: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn run(&mut self) {
        let barrier = Arc::new(Barrier::new(self.config.num_workers as usize + 1));
        let mut num_left = (self.config.num_games / self.config.job_size) + 1;
        let size = self.config.job_size;
        let book = Arc::new(Mutex::new(self.book.clone()));
        let pool = ThreadPool::new(self.config.num_workers as usize);
        while num_left != 0 {
            num_left -= 1;
            let output = self.output.clone();
            let inner_barrier = barrier.clone();
            let book = book.clone();
            pool.execute(move || {
                let my_job = book.lock().unwrap().generate_job(size);
                let mut gathered = vec![];
                // run the job

                // add all of the gathered games to the output
                {
                    let mut output = output.lock().unwrap();
                    output.append(&mut gathered);
                }
                // wait
                inner_barrier.wait();
            });
            barrier.wait();
        }
    }
}

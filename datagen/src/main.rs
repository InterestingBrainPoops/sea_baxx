use std::{
    fs::{self, File},
    io::{self, BufRead},
    path::Path,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Instant,
};

use clap::{Args, Parser, Subcommand};
use datagen::{datacollector::DataCollector, runner::Runner, Data, Openings};
use game::board::Board;
use serde::Deserialize;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    Generate(GenerateArgs),
    Information(InformationArgs),
}
#[derive(Args)]
struct InformationArgs {
    #[arg(short, long)]
    data_path: String,
}
#[derive(Args)]
struct GenerateArgs {
    /// Name of the person to greet
    #[arg(short, long)]
    openings_path: String,

    /// Amount of data to generate, in terms of games
    #[arg(short, long, default_value_t = 1000)]
    amount: u32,

    /// Amount of concurrency to use
    #[arg(short, long, default_value_t = 3)]
    concurrency: u8,

    /// Path to write the generated data to
    #[arg(short, long)]
    to_write: String,
}

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Generate(args) => {
            // grab all openings
            let mut openings = Openings { openings: vec![] };
            if let Ok(lines) = read_lines(&args.openings_path) {
                // Consumes the iterator, returns an (Optional) String
                for line in lines.flatten() {
                    openings.openings.push(Board::new(line));
                }
            } else {
                panic!("Openings not found at path {}", args.openings_path);
            }

            // depth to reach: 7 (chosen because we can hit this depth within ~100 ms rn)
            // architecture:
            // N runner nodes that each have two search instances, engine 1 and engine 2. since both engines are the same, it doesn't really matter all that much.
            // 1 data collector node that takes in game information like game length, and game score. Games are stored as the initial position + all the moves that follow. We also store the final position of the game.
            // Each runner gets a MPSC handle to send data into the data collector thread, which spins while waiting for data to store.
            // Amongst all of them is going to be an atomic int for read write, which stores the number of games left to run.

            let num_games_left = Arc::new(Mutex::new(args.amount));
            let (send, recv) = mpsc::channel();
            let dc_games_left = num_games_left.clone();
            let done_writing = Arc::new(Mutex::new(false));
            let done_writing_thread = done_writing.clone();

            thread::spawn(move || {
                let mut datacollector = DataCollector::new();
                datacollector.run(dc_games_left, recv);
                datacollector.write(args.to_write);
                println!("Written");
                *done_writing_thread.lock().unwrap() = true;
            });
            for id in 0..args.concurrency {
                let x = openings.clone();
                let y = num_games_left.clone();
                let s = send.clone();
                thread::spawn(move || {
                    let mut runner = Runner::new(x);
                    runner.start(id as usize, y, s);
                });
            }
            let mut old = *num_games_left.lock().unwrap();
            let t0 = Instant::now();
            while *num_games_left.lock().unwrap() != 0 {
                let num_games_left = *num_games_left.lock().unwrap();
                if num_games_left % 10 == 0 && old != num_games_left {
                    old = num_games_left;
                    println!("{}", num_games_left);
                    println!(
                        "Estimated time left: {:?}",
                        ((Instant::now() - t0) / (args.amount - old)) * old
                    );
                }
            }
            let time_taken = Instant::now() - t0;
            println!(
                "Time taken {:?}, Time per game: {:?}",
                time_taken,
                time_taken / args.amount
            );
            while !*done_writing.lock().unwrap() {}
        }
        Commands::Information(args) => {
            let thing = fs::read(Path::new(&args.data_path)).unwrap();
            // deserialize the file from messagepack to the Frames struct
            let thing2 =
                <Data>::deserialize(&mut rmp_serde::Deserializer::new(&thing[..])).unwrap();
            println!("# of games: {}", thing2.boards.len());
            println!(
                "# of positions: {}",
                thing2
                    .boards
                    .iter()
                    .map(|x| x.moves.len() + 1)
                    .sum::<usize>()
            );
            println!("Turn count statistics:");
            println!(
                "   average: {}",
                thing2
                    .boards
                    .iter()
                    .map(|x| x.moves.len() + 1)
                    .sum::<usize>() as f64
                    / thing2.boards.len() as f64
            );

            println!(
                "   min: {}",
                thing2
                    .boards
                    .iter()
                    .map(|x| x.moves.len() + 1)
                    .min()
                    .unwrap()
            );
            println!(
                "   max: {}",
                thing2
                    .boards
                    .iter()
                    .map(|x| x.moves.len() + 1)
                    .max()
                    .unwrap()
            );
        }
    }
}

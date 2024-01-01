use std::{
    fs,
    sync::{mpsc::Receiver, Arc, Mutex},
};

use serde::Serialize;

use crate::{Data, Game};

pub struct DataCollector {
    data: Data,
}

impl DataCollector {
    pub fn new() -> DataCollector {
        DataCollector {
            data: Data { boards: vec![] },
        }
    }

    pub fn run(&mut self, num_games_left: Arc<Mutex<u32>>, reciever: Receiver<Game>) {
        while *num_games_left.lock().unwrap() != 0 {
            let game = reciever.recv().unwrap();
            self.data.boards.push(game);
        }
    }

    pub fn write(&self, path: String) {
        let mut buf = vec![];
        // serialize this into the buf
        self.data
            .serialize(&mut rmp_serde::Serializer::new(&mut buf))
            .unwrap();
        // write the buffer into the datastore file
        fs::write(path, buf).expect("unable to write to ./datastore");
    }
}

use std::ops::{Index, IndexMut};

use game::{board::Board, movegen::Move};

pub struct Table {
    entries: Vec<Option<Entry>>,
}

impl Table {
    pub fn new(size: usize) -> Table {
        let mut entries = vec![];
        for _ in 0..size {
            entries.push(None);
        }
        Table { entries: entries }
    }

    pub fn reset(&mut self) {
        self.entries.iter_mut().for_each(|x| *x = None);
    }
}
impl IndexMut<&Board> for Table {
    fn index_mut(&mut self, index: &Board) -> &mut Self::Output {
        let num_entries = self.entries.len();
        &mut self.entries[(index.zobrist_hash() % num_entries as u64) as usize]
    }
}

impl Index<&Board> for Table {
    type Output = Option<Entry>;

    fn index(&self, index: &Board) -> &Self::Output {
        &self.entries[(index.zobrist_hash() % self.entries.len() as u64) as usize]
    }
}

pub struct Entry {
    pub hash: u64,
    pub hash_move: Move,
    pub score: i32,
    pub depth: u8,
    pub node_type: NodeType,
}
#[derive(PartialEq, Eq)]
pub enum NodeType {
    Upper,
    Lower,
    Exact,
}

impl Entry {
    pub fn new(hash: u64, hash_move: Move, score: i32, depth: u8, node_type: NodeType) -> Entry {
        Entry {
            hash,
            hash_move,
            score,
            depth,
            node_type,
        }
    }
}

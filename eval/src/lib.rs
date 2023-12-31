use game::{board::Board, movegen::singles};

pub struct Eval {}

impl Eval {
    pub fn new() -> Self {
        Eval {}
    }

    pub fn evaluate(&self, board: &Board) -> i32 {
        let us = board.boards[board.side_to_move as usize];
        let them = board.boards[1 - board.side_to_move as usize];
        let piece_count = us.count_ones() as i32 - them.count_ones() as i32;
        let mobilty =
            (singles(us) & !us).count_ones() as i32 - (singles(them) & !them).count_ones() as i32;

        piece_count
    }
}

use crate::movegen::Move;

pub struct MovePicker {
    moves: Vec<Move>,
    hash_move: Option<Move>,
    killer_move: Option<Move>,
}

impl MovePicker {
    pub fn new(moves: Vec<Move>, hash_move: Option<Move>, killer_move: Option<Move>) -> MovePicker {
        MovePicker {
            moves,
            hash_move,
            killer_move,
        }
    }

    pub fn sort(&self) -> Vec<Move> {
        let mut out = vec![];
        if let Some(killer) = self.killer_move {
            out.push(killer);
        }
        if let Some(hash_move) = self.hash_move {
            out.push(hash_move);
        }

        let mut new_moves = self
            .moves
            .iter()
            .filter(|x| !out.contains(x))
            .cloned()
            .collect::<Vec<Move>>();
        new_moves.sort_by_key(|x| {
            let mut key = 0;

            key += 8 - x.capture_square.count_ones() as i32;

            if x.from != 0 {
                key += 8
            }
            key
        });
        out.append(&mut new_moves);

        out
    }
}

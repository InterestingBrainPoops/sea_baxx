use std::fmt::Display;

use crate::board::{BitBoard, Board, State};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Move {
    pub null: bool,
    pub from: BitBoard,
    pub to: BitBoard,
    pub capture_square: BitBoard,
}

impl Move {
    pub fn from_str(input: &str, all: u64) -> Self {
        if input.len() == 4 {
            let from = an_to_bb(input[0..2].to_string());
            let to = an_to_bb(input[2..4].to_string());
            let capture_square =
                (to << 1 | to >> 1 | to << 8 | to >> 8 | to << 9 | to >> 9 | to << 7 | to >> 7)
                    & all;
            Self {
                null: false,
                from,
                to,
                capture_square,
            }
        } else {
            let to = an_to_bb(input[0..2].to_string());

            let capture_square =
                (to << 1 | to >> 1 | to << 8 | to >> 8 | to << 9 | to >> 9 | to << 7 | to >> 7)
                    & all;
            Move {
                null: false,
                from: 0,
                to,
                capture_square,
            }
        }
    }
}

pub fn bb_to_an(bb: u64) -> String {
    let mut alph = ["a", "b", "c", "d", "e", "f", "g"];
    let mut num = [1, 2, 3, 4, 5, 6, 7];
    let ld_zero = bb.trailing_zeros();

    return format!(
        "{}{}",
        alph[(ld_zero % 8) as usize],
        num[(ld_zero / 8) as usize]
    );
}

pub fn an_to_bb(an: String) -> u64 {
    let mut alph = ["a", "b", "c", "d", "e", "f", "g"];
    let mut num = [1, 2, 3, 4, 5, 6, 7];

    return 1
        << (alph.iter().position(|r| *r == &an[0..1]).unwrap()
            + num
                .iter()
                .position(|r| *r == an[1..2].parse::<usize>().unwrap())
                .unwrap()
                * 8);
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.null {
            return write!(f, "0000");
        }
        if self.from == 0 {
            write!(f, "{}", bb_to_an(self.to))
        } else {
            write!(f, "{}{}", bb_to_an(self.from), bb_to_an(self.to))
        }
    }
}

pub fn generate_moves(board: &Board) -> Vec<Move> {
    if board.game_over() {
        return vec![];
    }
    let mut out = vec![];
    let mut my_pieces = board.current_pieces();
    // other persons pieces
    let other_pieces = board.other_pieces();
    // all of the blocking pieces
    let all_blockers = board.blockers();

    // generate singles
    //
    // this contains all of the possible single moves for that bitmask that are within the 7x7 board.
    let singles = singles(my_pieces);
    // find all of the single moves that move into an empty square
    let mut legal_singles = singles & (!all_blockers);
    // iterate through all legal single moves
    for _ in 0..legal_singles.count_ones() {
        // find the to mask for the single move
        let to_mask = 1 << legal_singles.trailing_zeros();
        // remove this from the total mask of single moves
        legal_singles ^= to_mask;
        // find the captures for this move
        let captures = (to_mask << 1
            | to_mask >> 1
            | to_mask << 8
            | to_mask >> 8
            | to_mask << 9
            | to_mask >> 9
            | to_mask << 7
            | to_mask >> 7)
            & other_pieces;

        out.push(Move {
            null: false,
            from: 0, // from mask doesnt matter for 1 moves since you dont remove the starting point
            to: to_mask,
            capture_square: captures,
        });
    }

    // iterate through each square for the side to move
    for _ in 0..my_pieces.count_ones() {
        // get the from bit mask
        let from_mask = 1 << my_pieces.trailing_zeros();

        my_pieces ^= from_mask;
        // generate 2 moves
        //
        // generates all of the possible double moves that are not moving oob
        let doubles = doubles(from_mask);

        // find all doubles that move into empty squares
        let mut legal_doubles = doubles & (!all_blockers);
        // iterate through all double moves
        for _ in 0..legal_doubles.count_ones() {
            // find the to mask for the double move
            let to_mask = 1 << legal_doubles.trailing_zeros();
            // remove this from the total mask of double moves
            legal_doubles ^= to_mask;
            // find the captures for this move
            let captures = (to_mask << 1
                | to_mask >> 1
                | to_mask << 8
                | to_mask >> 8
                | to_mask << 7
                | to_mask >> 7
                | to_mask << 9
                | to_mask >> 9)
                & other_pieces;

            out.push(Move {
                null: false,
                from: from_mask,
                to: to_mask,
                capture_square: captures,
            });
        }
    }

    if out.is_empty() {
        out.push(Move {
            null: true,
            from: 0,
            to: 0,
            capture_square: 0,
        })
    }

    out
}

pub fn singles(bb: u64) -> u64 {
    return (bb << 1 | bb >> 1 | bb << 8 | bb >> 8 | bb << 9 | bb >> 9 | bb << 7 | bb >> 7)
        & 0x7f7f7f7f7f7f7f_u64;
}

pub fn doubles(bb: u64) -> u64 {
    return (
        // right
        ((bb << 2 | bb << 10 | bb << 18 | bb >> 6 | bb >> 14 | bb << 17 | bb >> 15) & 0x7e7e7e7e7e7e7e) |
        // center
        ((bb << 16 | bb >> 16) & 0x7f7f7f7f7f7f7f) |
        // left
        ((bb >> 2 | bb >> 10 | bb >> 18 | bb << 6 | bb << 14 | bb << 15 | bb >> 17) & 0x3f3f3f3f3f3f3f)
    );
}

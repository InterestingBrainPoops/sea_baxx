use std::ops::Not;

use crate::movegen::singles;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Board {
    pub blockers: BitBoard,
    pub boards: [BitBoard; 2],
    pub side_to_move: Side,
    pub half_move: u8,
    pub full_move: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Black,
    White,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Winner,
    Loser,
    Draw,
    Ongoing,
}

impl From<&str> for Side {
    fn from(string: &str) -> Self {
        match string {
            "x" => Side::Black,
            "o" => Side::White,
            _ => {
                panic!(
                    "Failed string to Side conversion : Expected either x or o, got :{}",
                    string
                )
            }
        }
    }
}

impl Not for Side {
    type Output = Side;

    fn not(self) -> Self::Output {
        match self {
            Side::Black => Side::White,
            Side::White => Side::Black,
        }
    }
}

impl From<Side> for usize {
    fn from(side: Side) -> Self {
        match side {
            Side::Black => 0,
            Side::White => 1,
        }
    }
}

impl Side {
    pub fn as_bool(&self) -> bool {
        match self {
            Side::Black => false,
            Side::White => true,
        }
    }

    pub fn toi32(&self) -> i32 {
        match self {
            Side::Black => -1,
            Side::White => 1,
        }
    }
}

pub type BitBoard = u64;

impl Board {
    pub fn new(fen: String) -> Board {
        let mut out = Board {
            blockers: 0,
            boards: [0; 2],
            side_to_move: Side::Black,
            half_move: 0,
            full_move: 0,
        };

        let parts = fen.split(' ').collect::<Vec<&str>>();

        // get all the board info (blockers, player 1 board, player 2 board)

        // current board spot (starts at a8)
        let mut current_board_index = 48;
        // iteratate over board fen
        for text in parts[0].chars() {
            match text {
                'x' => {
                    out.boards[0] |= 1 << current_board_index;
                    current_board_index += 1;
                }
                'o' => {
                    out.boards[1] |= 1 << current_board_index;
                    current_board_index += 1;
                }
                '-' => {
                    out.blockers |= 1 << current_board_index;
                    current_board_index += 1;
                }
                '/' => {
                    current_board_index -= 15;
                }
                _ => {
                    let value = String::from(text)
                        .parse::<u8>()
                        .unwrap_or_else(|_| panic!("Invalid character {}", text));
                    current_board_index += value;
                }
            }
        }
        // get side to move
        out.side_to_move = Side::from(parts[1]);
        // get half move counter
        out.half_move = String::from(parts[2]).parse::<u8>().unwrap_or_else(|_| {
            panic!(
                "Unexpected substring when parsing half_move counter :{}",
                parts[2]
            )
        });
        // get full move counter
        out.full_move = String::from(parts[2]).parse::<u32>().unwrap_or_else(|_| {
            panic!(
                "Unexpected substring when parsing full move counter :{}",
                parts[3]
            )
        });
        out
    }

    pub fn game_over(&self) -> bool {
        let both = self.current_pieces() | self.other_pieces();
        let moves = singles(singles(both));

        if self.current_pieces() == 0 {
            return true;
        }

        if self.other_pieces() == 0 {
            return true;
        }

        if self.half_move >= 100 {
            return true;
        }

        if moves & self.empty() != 0 {
            return false;
        }

        true
    }
    pub fn status(&self) -> Status {
        let _current = self.side_to_move;
        let _other = !self.side_to_move;
        let both = self.current_pieces() | self.other_pieces();
        let moves = singles(singles(both));

        if self.current_pieces() == 0 {
            // side to move has no pieces
            return Status::Loser;
        }

        if self.other_pieces() == 0 {
            // other person has no pieces
            return Status::Winner;
        }

        if self.half_move >= 100 {
            // 50 move rule
            return Status::Draw;
        }

        if moves & self.empty() != 0 {
            // side to move still has moves left
            return Status::Ongoing;
        }

        if self.current_pieces().count_ones() > self.other_pieces().count_ones() {
            Status::Winner
        } else {
            Status::Loser
        }
    }
    pub fn current_pieces(&self) -> u64 {
        self.boards[self.side_to_move as usize]
    }

    pub fn other_pieces(&self) -> u64 {
        self.boards[1 - self.side_to_move as usize]
    }

    pub fn blockers(&self) -> u64 {
        self.boards[0] | self.boards[1] | self.blockers
    }

    pub fn empty(&self) -> u64 {
        !self.blockers()
    }

    pub fn hash(&self) -> u64 {
        self.boards[0] | self.boards[1]
    }
}

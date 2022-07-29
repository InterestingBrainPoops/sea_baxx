use crate::{
    board::{BitBoard, Board, Side},
    movegen::Move,
};

pub struct Delta {
    old_spot: BitBoard,
    captures: BitBoard,
}

pub fn make_move(board: &mut Board, mov: &Move) -> u8 {
    // if mov.to == 8 {
    //     panic!();
    // }
    if !mov.null {
        // add in the new stone for the side to move
        board.boards[board.side_to_move as usize] |= mov.to;

        // remove the old stone for the side to move , dont need an if here since from will be zero if its a single move
        board.boards[board.side_to_move as usize] ^= mov.from;

        // remove the capture squares from the other side (doesnt matter if its zero or not since xor is great)
        board.boards[1 - board.side_to_move as usize] ^= mov.capture_square;
        // add the captures to our side
        board.boards[board.side_to_move as usize] |= mov.capture_square;
    }
    // flip side to move
    board.side_to_move = !board.side_to_move;

    // increment full move counter iff side to move is black after flipping
    if board.side_to_move == Side::Black {
        board.full_move += 1;
    }

    let old_half_move = board.half_move;
    // update half move counter
    if mov.capture_square != 0 {
        board.half_move = 0;
    } else {
        board.half_move += 1;
    }

    old_half_move
}

pub fn unmake_move(board: &mut Board, mov: &Move, old_half_move: u8) {
    // update half move counter
    board.half_move = old_half_move;
    // decrement full move counter iff side to move is black before flipping
    if board.side_to_move == Side::Black {
        board.full_move -= 1;
    }

    // flip the side to move
    board.side_to_move = !board.side_to_move;

    if !mov.null {
        // add back the old from position (doesnt matter if the old stone is still there because of the OR operation truth table)
        board.boards[board.side_to_move as usize] |= mov.from;

        // remove the just moved to square
        board.boards[board.side_to_move as usize] ^= mov.to;

        // add back the captures (doesnt matter if capture squares are zero or not since OR is great)
        board.boards[1 - board.side_to_move as usize] |= mov.capture_square;
        // remove the captures from our side
        board.boards[board.side_to_move as usize] ^= mov.capture_square;
    }
}

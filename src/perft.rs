use crate::{
    board::Board,
    move_app::{make_move, unmake_move},
    movegen::generate_moves,
};

#[allow(dead_code)]
fn perft(board: &mut Board, depth: u8, max_depth: u8) -> u64 {
    let mut nodes = 0;
    if depth == 0 {
        return 1;
    }
    let mut counter = 0;
    for mov in &generate_moves(board) {
        let mut subtree_nodes = 0;
        let t1 = *board;
        let delta = make_move(board, mov);
        let inc = perft(board, depth - 1, max_depth);
        subtree_nodes += inc;
        nodes += inc;
        unmake_move(board, mov, delta);
        assert_eq!(board.clone(), t1);
        if depth == max_depth {
            counter += 1;
            println!("{}) {} {}", counter, mov, subtree_nodes);
            // println!("{:?}", mov);
        }
    }

    nodes
}

#[cfg(test)]
mod tests {

    use crate::{
        board::Board,
        move_app::make_move,
        movegen::{bb_to_an, generate_moves},
    };

    use super::perft;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
    #[test]
    fn an_test() {
        assert_eq!(bb_to_an(0x40000000000000_u64), "g7");

        assert_eq!(bb_to_an(0x80000_u64), "d3");
        assert_eq!(bb_to_an(0x40_u64), "g1");
        assert_eq!(bb_to_an(0x1000000000000_u64), "a7");
        assert_eq!(bb_to_an(0x1000_u64), "e2");
    }
    #[test]
    fn perft_all() {
        let tests = vec![
            (
                "x5o/7/7/7/7/7/o5x x 0 1",
                vec![1, 16, 256, 6460, 155888, 4752668],
            ),
            ("7/7/7/7/7/7/7 x 0 1", vec![1, 0, 0, 0, 0, 0]),
            ("7/7/7/7/7/7/7 o 0 1", vec![1, 0, 0, 0, 0, 0]),
            (
                "x5o/7/7/7/7/7/o5x o 0 1",
                vec![1, 16, 256, 6460, 155888, 4752668],
            ),
            (
                "x5o/7/2-1-2/7/2-1-2/7/o5x x 0 1",
                vec![1, 14, 196, 4184, 86528, 2266352],
            ),
            (
                "x5o/7/2-1-2/7/2-1-2/7/o5x o 0 1",
                vec![1, 14, 196, 4184, 86528, 2266352],
            ),
            (
                "x5o/7/2-1-2/3-3/2-1-2/7/o5x x 0 1",
                vec![1, 14, 196, 4100, 83104, 2114588],
            ),
            (
                "x5o/7/2-1-2/3-3/2-1-2/7/o5x o 0 1",
                vec![1, 14, 196, 4100, 83104, 2114588],
            ),
            (
                "x5o/7/3-3/2-1-2/3-3/7/o5x x 0 1",
                vec![1, 16, 256, 5948, 133264, 3639856],
            ),
            (
                "x5o/7/3-3/2-1-2/3-3/7/o5x o 0 1",
                vec![1, 16, 256, 5948, 133264, 3639856],
            ),
            (
                "7/7/7/7/ooooooo/ooooooo/xxxxxxx x 0 1",
                vec![1, 1, 75, 249, 14270, 452980],
            ),
            (
                "7/7/7/7/ooooooo/ooooooo/xxxxxxx o 0 1",
                vec![1, 75, 249, 14270, 452980],
            ),
            (
                "7/7/7/7/xxxxxxx/xxxxxxx/ooooooo x 0 1",
                vec![1, 75, 249, 14270, 452980],
            ),
            (
                "7/7/7/7/xxxxxxx/xxxxxxx/ooooooo o 0 1",
                vec![1, 1, 75, 249, 14270, 452980],
            ),
            (
                "7/7/7/2x1o2/7/7/7 x 0 1",
                vec![1, 23, 419, 7887, 168317, 4266992],
            ),
            (
                "7/7/7/2x1o2/7/7/7 o 0 1",
                vec![1, 23, 419, 7887, 168317, 4266992],
            ),
            ("x5o/7/7/7/7/7/o5x x 100 1", vec![1, 0, 0, 0, 0]),
            ("x5o/7/7/7/7/7/o5x o 100 1", vec![1, 0, 0, 0, 0]),
            (
                "7/7/7/7/-------/-------/x5o x 0 1",
                vec![1, 2, 4, 13, 30, 73, 174],
            ),
            (
                "7/7/7/7/-------/-------/x5o o 0 1",
                vec![1, 2, 4, 13, 30, 73, 174],
            ),
        ];

        for (idx, (fen, numbers)) in tests.iter().enumerate() {
            println!("fen: {}, idx : {}", fen, idx);
            perft_test(numbers.to_vec(), fen.to_string(), idx);
        }
    }

    #[test]
    fn game_end() {
        let white_win = "6o/7/2o4/3o3/4o2/7/o6 o 0 1";
        let board = Board::new(white_win.to_string());
        assert!(board.game_over());
    }

    fn perft_test(values: Vec<u64>, fen: String, idx: usize) {
        let mut board = Board::new(fen);
        for (depth, number) in values.iter().enumerate() {
            println!("depth: {}", depth);
            if depth == 60 && idx == 180 {
                let moves = generate_moves(&board);
                make_move(&mut board, &moves[0]);
                let moves = generate_moves(&board);
                make_move(&mut board, &moves[0]);
                let moves = generate_moves(&board);
                make_move(&mut board, &moves[0]);
                let moves = generate_moves(&board);
                make_move(&mut board, &moves[0]);
                let moves = generate_moves(&board);
                make_move(&mut board, &moves[0]);
                // let moves = generate_moves(&board);
                println!("{}", board.boards[0]);

                println!("Count: {}", moves.len());
                println!("Moves: {:?}", moves);
                let nodes = perft(&mut board, 2_u8, 2_u8);
                assert_eq!(*number, nodes);
            } else {
                let nodes = perft(&mut board, depth as u8, depth as u8);
                assert_eq!(*number, nodes);
            }
        }
    }
}

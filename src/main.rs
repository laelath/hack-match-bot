mod board;
mod screen;

use board::{Board, Item, Move};

use std::collections::{HashSet, VecDeque};
use std::thread;
use std::time::{Duration, Instant};

use x11rb::connection::Connection;

const MAX_SEARCH_TIME: Duration = Duration::from_millis(110);
const SOLVE_WAIT_TIME: Duration = Duration::from_millis(4 * screen::KEY_DELAY_MILLIS + 12);

fn find_match(start: &Board) -> Vec<Move> {
    if start.has_match() {
        println!("Board already has an unrealized match");
        return vec![];
    }

    let start_time = Instant::now();

    let mut boards = VecDeque::with_capacity(10000);
    let mut seen = HashSet::with_capacity(80000);

    let mut highest_score = start.score();
    let mut highest_path = vec![];

    let mut explored = 1;
    let mut steps_ahead = 0;

    seen.insert(start.clone());
    boards.push_back((start.clone(), vec![]));

    while !boards.is_empty() {
        if Instant::now().duration_since(start_time) > MAX_SEARCH_TIME {
            if highest_path.len() == 0 {
                println!("Search timed out, could not find a match or better board");
            } else {
                println!("Search timed out, defaulting to highest score");
            }
            println!(
                "Explored {} boards, {} moves deep, returning path {} long",
                explored,
                steps_ahead,
                highest_path.len()
            );
            return highest_path;
        }

        let (board, path) = boards.pop_front().unwrap();
        if path.len() > steps_ahead {
            steps_ahead = path.len() + 1;
        }

        for m in [Move::Left, Move::Right, Move::Swap, Move::Exchange].iter() {
            let new_board = board.do_move(*m);
            if !seen.contains(&new_board) {
                explored += 1;

                // Create the path to this board
                let mut new_path = path.clone();
                new_path.push(*m);

                // check if the board has a match on it
                if new_board.has_match() {
                    println!("Found match");
                    println!(
                        "Explored {} boards, {} moves deep, returning path {} long",
                        explored,
                        steps_ahead + 1,
                        new_path.len()
                    );
                    return new_path;
                }

                // Add the board to the seen list
                seen.insert(new_board.clone());

                // check if the board has a higher score
                let new_score = new_board.score();
                if new_score > highest_score {
                    highest_score = new_score;
                    highest_path = new_path.clone();
                }

                // Push the board onto the worklist
                boards.push_back((new_board, new_path));
            }
        }
    }

    // panic!("find_match() cannot be called on a board with no matches");
    if highest_path.len() == 0 {
        println!("Exhausted search, could not find a match or better board");
    } else {
        println!("Exhausted search, defaulting to highest score");
    }
    println!(
        "Explored {} boards, {} moves deep, returning path {} long",
        explored,
        steps_ahead,
        highest_path.len()
    );
    highest_path
}

fn main() {
    let (conn, screen_num) = x11rb::connect(None).unwrap();
    let setup = conn.setup();
    let screen = &setup.roots[screen_num];

    let keycodes = screen::get_keycodes(&conn, &setup);

    println!("{:?}", keycodes);

    println!("Finding EXAPUNKS window");
    let window = match screen::get_exapunks_window(&conn, screen.root) {
        Some(window) => window,
        None => panic!("Unable to find EXAPUNKS window."),
    };

    println!("Validating window parameters");
    screen::validate_window(&conn, &setup, &screen, window);

    screen::activate_window(&conn, window);

    let mut board = board::make_board(
        0,
        Item::Empty,
        [[Item::Empty; board::MAX_COLS]; board::MAX_ROWS],
    );
    let mut generation = 0;

    loop {
        board = screen::get_new_board(&conn, window, &board);

        println!("Generation: {}", generation);
        board.print();
        println!("Solving board");
        let path = find_match(&board);
        println!("Playing path {:?}", path);
        screen::play_path(&conn, &keycodes, path);
        generation += 1;
        println!();
        thread::sleep(SOLVE_WAIT_TIME);
    }
}

mod board;
mod screen;

#[macro_use(defer)]
extern crate scopeguard;

use board::{Board, Move};

use std::collections::{HashSet, VecDeque};
use std::thread::sleep;
// use std::time::Duration;
use std::time::{Duration, Instant};
// use self::SearchError::*;

const MAX_SEARCH_TIME: Duration = Duration::from_millis(1000);
const BOARD_SOLVE_WAIT: Duration = Duration::from_millis(4 * screen::KEY_DELAY_MILLIS + 3);
// const EXPLORE_LIMIT: usize = 1000000000;

// enum SearchError {
//     Timeout,
//     NoMatch,
// }

fn find_match(start: &Board) -> Option<Vec<Move>> {
    let start_time = Instant::now();

    let mut boards = VecDeque::new();
    let mut seen = HashSet::new();

    seen.insert(start.clone());
    boards.push_back((start.clone(), vec![]));

    while !boards.is_empty() {
        let (board, path) = boards.pop_front().unwrap();

        if board.has_match() {
            return Some(path);
        }

        if Instant::now().duration_since(start_time) > MAX_SEARCH_TIME {
            return None;
        }

        for m in [Move::Left, Move::Right, Move::Swap, Move::Exchange].iter() {
            let new_board = board.do_move(*m);
            if !seen.contains(&new_board) {
                seen.insert(new_board.clone());

                let mut new_path = path.clone();
                new_path.push(*m);

                boards.push_back((new_board, new_path));
            }
        }
    }
    panic!("find_match() cannot be called on a board with no matches");
}

fn main() {
    let display = unsafe { x11::xlib::XOpenDisplay(std::ptr::null()) };
    assert!(!display.is_null(), "Failed to get display");
    defer! {{ unsafe { x11::xlib::XCloseDisplay(display); } }}

    println!("Getting EXAPUNKS window");
    let window = screen::get_exapunks_window(display);

    println!("Validating window parameters");
    screen::validate_window(display, window);

    screen::activate_window(display, window);

    let mut prev_board = board::make_board(0, None, [[None; board::MAX_COLS]; board::MAX_ROWS]);
    let mut generation = 0;

    loop {
        match screen::get_board_from_window(display, window) {
            Some(board) => {
                if board != prev_board {
                    println!("Generation: {}", generation);
                    board.print();
                    if board.imbalance() >= 4 {
                        println!("Balancing board");
                        screen::play_path(display, board.solve_imbalance());
                    } else if board.can_make_match() {
                        println!("Solving board");
                        match find_match(&board) {
                            Some(path) => {
                                println!("Playing path");
                                match screen::get_board_from_window(display, window) {
                                    Some(new_board) => {
                                        screen::play_trim_path(display, path, &new_board)
                                    }
                                    None => screen::play_path(display, path),
                                }
                            }
                            None => println!("Search timed out"),
                        }
                    } else {
                        println!("Waiting");
                    }
                    // match find_match(&board) {
                    //     Some(path) => screen::play_path(display, path),
                    //     None => println!("No matches found"),
                    // }
                    prev_board = board;
                    generation += 1;
                    println!();
                }
            }
            None => (),
        };
        sleep(BOARD_SOLVE_WAIT);
    }
}

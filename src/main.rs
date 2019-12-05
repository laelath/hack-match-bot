mod board;
mod screen;

#[macro_use(defer)]
extern crate scopeguard;

use board::{Board, Item, Move};

use std::collections::{HashSet, VecDeque};
use std::thread;
use std::time::{Duration, Instant};

use x11::xlib;
use x11::xlib::{Display, Window};

const MAX_SEARCH_TIME: Duration = Duration::from_millis(110);
const SOLVE_WAIT_TIME: Duration = Duration::from_millis(4 * screen::KEY_DELAY_MILLIS + 12);
const RECHECK_WAIT_TIME: Duration = Duration::from_millis(screen::KEY_DELAY_MILLIS + 3);
const SCREEN_READ_FAIL_LIMIT: usize = 25;

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
        /*if Instant::now().duration_since(check_time) >= BOARD_SOLVE_WAIT {
            match screen::get_board_from_window(display, window) {
                Some(screen_board) => if *start != screen_board {
                    println!("Search timed out, playing highest score");
                    return highest_path;
                }
                None => check_time = Instant::now(),
            }
        }*/

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

// checks that the board is seen twice in a row and is different from the given board
fn get_new_board(display: *mut Display, window: Window, prev_board: &mut Board) {
    // let mut failed = 0;
    loop {
        match screen::get_board_from_window(display, window) {
            Some(board) => {
                if board != *prev_board {
                    *prev_board = board;
                    break;
                }
            }
            None => {
                // if failed >= SCREEN_READ_FAIL_LIMIT {
                //     println!("Failed to read screen too many times, exiting");
                //     std::process::exit(0);
                // }
                // failed += 1;
                thread::sleep(RECHECK_WAIT_TIME);
            }
        }
    }

    /*
    loop {
        thread::sleep(BOARD_SOLVE_WAIT);
        match screen::get_board_from_window(display, window) {
            Some(board) => {
                if board == *prev_board {
                    break;
                }
                *prev_board = board;
            }
            None => {
                if failed >= SCREEN_READ_FAIL_LIMIT {
                    println!("Failed to read screen too many times, exiting");
                    std::process::exit(0);
                }
                failed += 1;
            }
        }
        thread::sleep(BOARD_SOLVE_WAIT);
    }
    */
}

fn main() {
    let display = unsafe { xlib::XOpenDisplay(std::ptr::null()) };
    assert!(!display.is_null(), "Failed to get display");
    defer! {{ unsafe { xlib::XCloseDisplay(display); } }}

    println!("Finding EXAPUNKS window");
    let window = screen::get_exapunks_window(display);

    println!("Validating window parameters");
    screen::validate_window(display, window);

    screen::activate_window(display, window);

    let mut board = board::make_board(
        0,
        Item::Empty,
        [[Item::Empty; board::MAX_COLS]; board::MAX_ROWS],
    );
    let mut generation = 0;

    loop {
        get_new_board(display, window, &mut board);

        println!("Generation: {}", generation);
        board.print();
        println!("Solving board");
        let path = find_match(&board);
        println!("Playing path {:?}", path);
        screen::play_path(display, path);
        generation += 1;
        println!();
        thread::sleep(SOLVE_WAIT_TIME);
    }
}

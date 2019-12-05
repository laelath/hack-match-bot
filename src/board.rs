use self::Color::*;
use self::Item::*;
use self::Move::*;
use std::fmt;
use std::mem;

pub const MAX_COLS: usize = 7;
pub const MAX_ROWS: usize = 9;

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub enum Color {
    Red,
    Yellow,
    Blue,
    Cyan,
    Pink,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub enum Item {
    File(Color),
    Bomb(Color),
    MatchedFile(Color),
    MatchedBomb(Color),
    Empty,
}

impl Item {
    pub fn is_matched(&self) -> bool {
        match self {
            File(_) => false,
            Bomb(_) => false,
            MatchedFile(_) => true,
            MatchedBomb(_) => true,
            Empty => false,
        }
    }

    pub fn to_matched(&self) -> Item {
        match self {
            File(c) => MatchedFile(*c),
            Bomb(c) => MatchedBomb(*c),
            MatchedFile(c) => MatchedFile(*c),
            MatchedBomb(c) => MatchedBomb(*c),
            Empty => Empty,
        }
    }

    pub fn to_normal(&self) -> Item {
        match self {
            File(c) => File(*c),
            Bomb(c) => Bomb(*c),
            MatchedFile(c) => File(*c),
            MatchedBomb(c) => Bomb(*c),
            Empty => Empty,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Red => write!(f, "r"),
            Yellow => write!(f, "y"),
            Blue => write!(f, "b"),
            Cyan => write!(f, "c"),
            Pink => write!(f, "p"),
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            File(c) => write!(f, "{}", c),
            Bomb(c) => write!(f, "{}", c.to_string().to_uppercase()),
            MatchedFile(_) => write!(f, "M"),
            MatchedBomb(_) => write!(f, "B"),
            Empty => write!(f, " "),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Board {
    phage_col: usize,
    held: Item,
    blocks: [[Item; MAX_COLS]; MAX_ROWS],
}

#[derive(Copy, Clone, Debug)]
pub enum Move {
    Left,
    Right,
    Exchange,
    Swap,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Left => write!(f, "left"),
            Right => write!(f, "right"),
            Exchange => write!(f, "exchange"),
            Swap => write!(f, "swap"),
        }
    }
}

impl Board {
    pub fn do_move(&self, m: Move) -> Board {
        let mut b = self.clone();

        match m {
            Left => b.move_left(),
            Right => b.move_right(),
            Exchange => b.exchange_held(),
            Swap => b.swap_blocks(),
        }

        b
    }

    fn move_left(&mut self) {
        if self.phage_col > 0 {
            self.phage_col -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.phage_col < 6 {
            self.phage_col += 1;
        }
    }

    fn exchange_held(&mut self) {
        if self.held == Empty {
            // Find a block to grab
            for row in (0..MAX_ROWS).rev() {
                if self.blocks[row][self.phage_col] != Empty {
                    if !self.blocks[row][self.phage_col].is_matched() {
                        mem::swap(&mut self.held, &mut self.blocks[row][self.phage_col]);
                    }
                    break;
                }
            }
        } else if self.blocks[MAX_ROWS - 1][self.phage_col] == Empty {
            // Try and place held block
            for row in (0..MAX_ROWS - 1).rev() {
                if self.blocks[row][self.phage_col] != Empty {
                    mem::swap(&mut self.held, &mut self.blocks[row + 1][self.phage_col]);
                    return;
                }
            }
            // If column empty, place on the bottom row
            mem::swap(&mut self.held, &mut self.blocks[0][self.phage_col]);
        }
    }

    fn swap_blocks(&mut self) {
        for row in (1..MAX_ROWS).rev() {
            if self.blocks[row][self.phage_col] != Empty {
                if !self.blocks[row][self.phage_col].is_matched()
                    && self.blocks[row - 1][self.phage_col] != Empty
                    && !self.blocks[row - 1][self.phage_col].is_matched()
                {
                    let (a, b) = self.blocks.split_at_mut(row);
                    mem::swap(&mut a[row - 1][self.phage_col], &mut b[0][self.phage_col]);
                }
                break;
            }
        }
    }

    fn has_matched(&self) -> bool {
        for row in 0..MAX_ROWS {
            for col in 0..MAX_COLS {
                if self.blocks[row][col].is_matched() {
                    return true;
                }
            }
        }
        false
    }

    fn settle_blocks(&mut self) {
        if self.has_matched() {
            return;
        }

        for col in 0..MAX_COLS {
            for row in 1..MAX_ROWS {
                for srow in (1..row + 1).rev() {
                    if self.blocks[srow - 1][col] == Empty && self.blocks[srow][col] != Empty {
                        let (a, b) = self.blocks.split_at_mut(srow);
                        mem::swap(&mut a[srow - 1][col], &mut b[0][col]);
                    }
                }
            }
        }
    }

    // group size ignores the matched flag, this allows for adding to matches
    fn group_size(
        &self,
        row: usize,
        col: usize,
        b: Item,
        mut visited: &mut [[bool; MAX_COLS]; MAX_ROWS],
    ) -> usize {
        if visited[row][col] {
            return 0;
        }

        if b != self.blocks[row][col].to_normal() {
            return 0;
        }

        visited[row][col] = true;

        let mut size = 1;

        if row > 0 {
            size += self.group_size(row - 1, col, b, &mut visited);
        }
        if col > 0 {
            size += self.group_size(row, col - 1, b, &mut visited);
        }
        if row < MAX_ROWS - 1 {
            size += self.group_size(row + 1, col, b, &mut visited);
        }
        if col < MAX_COLS - 1 {
            size += self.group_size(row, col + 1, b, &mut visited);
        }

        size
    }

    pub fn has_match(&self) -> bool {
        let mut visited: [[bool; MAX_COLS]; MAX_ROWS] = [[false; MAX_COLS]; MAX_ROWS];

        for row in 0..MAX_ROWS {
            for col in 0..MAX_COLS {
                let b = self.blocks[row][col];
                if b != Empty && !b.is_matched() {
                    let group_size = self.group_size(row, col, b, &mut visited);
                    let match_size = match b {
                        File(_) => 4,
                        Bomb(_) => 2,
                        MatchedFile(_) => panic!("Should not start match search from matched item"),
                        MatchedBomb(_) => panic!("Should not start match search from matched item"),
                        Empty => panic!("Empty items cannot be matched"),
                    };

                    if group_size >= match_size {
                        return true;
                    }
                }
            }
        }

        false
    }

    /*
    fn tallest_col(&self) -> (usize, usize) {
        let mut max = 0;
        let mut max_col = 0;
        for col in 0..MAX_COLS {
            for row in 0..MAX_ROWS {
                if self.blocks[row][col] == Empty {
                    if row > max {
                        max = row;
                        max_col = col;
                    }
                    break;
                }
            }
        }
        (max, max_col)
    }
    */

    /*
    fn shortest_col(&self) -> (usize, usize) {
        let mut min = MAX_COLS;
        let mut min_col = 0;
        for col in 0..MAX_COLS {
            for row in 0..MAX_ROWS {
                if self.blocks[row][col] == Empty {
                    if row < min {
                        min = row;
                        min_col = col;
                    }
                    break;
                }
            }
        }
        (min, min_col)
    }
    */

    fn column_heights(&self) -> [usize; MAX_COLS] {
        let mut heights = [0; MAX_COLS];
        for col in 0..MAX_COLS {
            for row in 0..MAX_ROWS {
                let b = self.blocks[row][col];
                if b != Empty {
                    heights[col] += 1;
                }
            }
        }
        heights
    }

    // board imbalance is the sum of squares of differences from the mean column height
    fn imbalance(&self) -> f64 {
        let heights = self.column_heights();

        let sum: usize = heights.iter().sum();
        let mean = sum as f64 / heights.len() as f64;

        heights.iter().map(|h| (*h as f64 - mean).powi(2)).sum()
    }

    pub fn score(&self) -> f64 {
        let mut score = 0.0;
        let mut visited = [[false; MAX_COLS]; MAX_ROWS];

        for row in 0..MAX_ROWS {
            for col in 0..MAX_COLS {
                if !visited[row][col] {
                    let b = self.blocks[row][col];
                    if b != Empty {
                        let group_size = self.group_size(row, col, b, &mut visited);
                        score += (group_size.pow(2)) as f64;
                    }
                }
            }
        }

        // Add one if holding a block so it doesn't prefer placing it
        if self.held != Empty {
            score += 1.0;
        }

        // let (max, _) = self.tallest_col();
        // let (min, _) = self.shortest_col();
        // assert!(min <= max);
        // score -= ((max - min).pow(2)) as f64;

        score -= self.imbalance().powi(2);
        // score -= max as f64;

        score
    }

    pub fn print(&self) {
        for row in self.blocks[..].iter() {
            print!("|");
            for item in row {
                print!("{}", item);
            }
            println!("|");
        }

        println!(
            "|{}^{}|",
            " ".repeat(self.phage_col),
            " ".repeat(MAX_COLS - self.phage_col - 1)
        );
        println!(
            "|{}{}{}|",
            " ".repeat(self.phage_col),
            self.held,
            " ".repeat(MAX_COLS - self.phage_col - 1)
        );
    }
}

pub fn make_board(phage_col: usize, held: Item, items: [[Item; MAX_COLS]; MAX_ROWS]) -> Board {
    let mut board = Board {
        phage_col: phage_col,
        held: held,
        blocks: items,
    };

    board.settle_blocks();

    board
}

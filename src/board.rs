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
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Board {
    phage_col: usize,
    held: Option<Item>,
    blocks: [[Option<Item>; MAX_COLS]; MAX_ROWS],
}

#[derive(Copy, Clone)]
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
        if self.held == None {
            // Find a block to grab
            for row in (0..MAX_ROWS).rev() {
                if self.blocks[row][self.phage_col] != None {
                    mem::swap(&mut self.held, &mut self.blocks[row][self.phage_col]);
                    break;
                }
            }
        } else {
            // Try and place held block
            for row in 0..MAX_ROWS {
                if self.blocks[row][self.phage_col] == None {
                    mem::swap(&mut self.held, &mut self.blocks[row][self.phage_col]);
                    break;
                }
            }
        }
    }

    fn swap_blocks(&mut self) {
        for row in (1..MAX_ROWS).rev() {
            if self.blocks[row][self.phage_col] != None {
                let (a, b) = self.blocks.split_at_mut(row);
                mem::swap(&mut a[row - 1][self.phage_col], &mut b[0][self.phage_col]);
                break;
            }
        }
    }

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

        if self.blocks[row][col] != Some(b) {
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
                match self.blocks[row][col] {
                    Some(b) => {
                        let group_size = self.group_size(row, col, b, &mut visited);
                        let match_size = match b {
                            File(_) => 4,
                            Bomb(_) => 2,
                        };

                        if group_size >= match_size {
                            return true;
                        }
                    }
                    None => {}
                }
            }
        }

        false
    }

    fn count_blocks(&self, item: Item) -> usize {
        let count = self
            .blocks
            .iter()
            .flat_map(|r| r.iter())
            .filter(|e| **e == Some(item))
            .count();

        match self.held {
            Some(held) => {
                if held == item {
                    count + 1
                } else {
                    count
                }
            }
            None => count,
        }
    }

    pub fn can_make_match(&self) -> bool {
        for color in [Red, Yellow, Blue, Cyan, Pink].iter() {
            if self.count_blocks(File(*color)) >= 4 {
                return true;
            } else if self.count_blocks(Bomb(*color)) >= 2 {
                return true;
            }
        }

        false
    }

    fn tallest_col(&self) -> (usize, usize) {
        let mut max = 0;
        let mut max_col = 0;
        for col in 0..MAX_COLS {
            for row in 0..MAX_ROWS {
                match self.blocks[row][col] {
                    Some(_) => (),
                    None => {
                        if row > max {
                            max = row;
                            max_col = col;
                        }
                        break;
                    }
                }
            }
        }
        (max, max_col)
    }

    fn shortest_col(&self) -> (usize, usize) {
        let mut min = MAX_COLS;
        let mut min_col = 0;
        for col in 0..MAX_COLS {
            for row in 0..MAX_ROWS {
                match self.blocks[row][col] {
                    Some(_) => (),
                    None => {
                        if row < min {
                            min = row;
                            min_col = col;
                        }
                        break;
                    }
                }
            }
        }
        (min, min_col)
    }

    pub fn imbalance(&self) -> usize {
        let (min, _) = self.shortest_col();
        let (max, _) = self.tallest_col();

        max - min
    }

    pub fn solve_imbalance(&self) -> Vec<Move> {
        let (_, min_col) = self.shortest_col();
        let (_, max_col) = self.tallest_col();

        let mut path = vec!();
        let mut curr_col = self.phage_col;

        if self.held.is_none() {
            // pick block from tallest column
            if curr_col < max_col {
                path.append(&mut vec![Move::Right; max_col - curr_col]);
            } else if curr_col > max_col {
                path.append(&mut vec![Move::Left; curr_col - max_col]);
            }

            path.push(Move::Exchange);
            curr_col = max_col;
        }

        // place block on smallest column
        if curr_col < min_col {
            path.append(&mut vec![Move::Right; min_col - curr_col]);
        } else if curr_col > min_col {
            path.append(&mut vec![Move::Left; curr_col - min_col]);
        }

        path.push(Move::Exchange);

        path
    }

    pub fn print(&self) {
        for row in self.blocks[..].iter() {
            print!(" ");
            for opt in row {
                match opt {
                    Some(b) => print!("{}", b),
                    None => print!(" "),
                }
            }
            println!();
        }

        for _ in 0..self.phage_col {
            print!(" ");
        }

        match &self.held {
            Some(b) => print!("<{}>", b),
            None => print!("<->"),
        }

        for _ in self.phage_col..MAX_COLS {
            print!(" ");
        }

        println!();
    }
}

pub fn make_board(
    phage_col: usize,
    held: Option<Item>,
    items: [[Option<Item>; MAX_COLS]; MAX_ROWS],
) -> Board {
    Board {
        phage_col: phage_col,
        held: held,
        blocks: items,
    }
}

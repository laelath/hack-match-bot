use crate::board;
use crate::board::{Board, Color, Item, Move};
use std::{thread, time};

use x11rb::connection::RequestConnection;
use x11rb::image::*;
use x11rb::protocol::xproto::*;
use x11rb::protocol::xtest;

pub const KEY_DELAY_MILLIS: u64 = 17;
const KEY_DELAY: time::Duration = time::Duration::from_millis(KEY_DELAY_MILLIS);
const RECHECK_WAIT_TIME: time::Duration = time::Duration::from_millis(KEY_DELAY_MILLIS + 3);

const ITEM_SIZE: usize = 72;
const BOARD_PIXEL_WIDTH: usize = board::MAX_COLS * ITEM_SIZE;
const BOARD_PIXEL_HEIGHT: usize = 770;
const BOARD_X_OFFSET: usize = 440;
const BOARD_Y_OFFSET: usize = 151;
const BOARD_PIXEL_HEIGHT_ITEMS: usize = 810 - BOARD_Y_OFFSET;

const PIXEL_X_OFFSET: usize = 30;
const PIXEL_MATCH_OFFSET: usize = 507 - BOARD_X_OFFSET - PIXEL_X_OFFSET;

const WINDOW_WIDTH: u16 = 1920;
const WINDOW_HEIGHT: u16 = 1080;

// verified by validate_window()
const BYTES_PER_PIXEL: usize = 4;

const XK_A: u32 = 0x0061;
const XK_D: u32 = 0x0064;
const XK_J: u32 = 0x006a;
const XK_K: u32 = 0x006b;

const YELLOW_BOMB_PIXEL: [u8; 4] = [7, 27, 29, 0];
const CYAN_BOMB_PIXEL: [u8; 4] = [45, 40, 3, 0];
const RED_BOMB_PIXEL: [u8; 4] = [15, 9, 66, 0];
const PINK_BOMB_PIXEL: [u8; 4] = [50, 0, 60, 0];
const BLUE_BOMB_PIXEL: [u8; 4] = [51, 4, 9, 0];

const PHAGE_HELD_Y_OFFSET: usize = 908 - BOARD_Y_OFFSET;
const PHAGE_PINK_DATA_X_OFFSET: usize = 31;
const PHAGE_PINK_DATA_Y_OFFSET: usize = 886 - BOARD_Y_OFFSET;
const PHAGE_SILVER_DATA_X_OFFSET: usize = 22;
const PHAGE_SILVER_DATA_Y_OFFSET: usize = 833 - BOARD_Y_OFFSET;
const PHAGE_CROUCH_X_OFFSET: usize = 3;
const PHAGE_CROUCH_Y_OFFSET: usize = 9;

const YELLOW_DATA: [u8; 32] = [
    24, 163, 235, 0, 24, 163, 235, 0, 24, 163, 235, 0, 24, 163, 235, 0, 24, 163, 235, 0, 24, 163,
    235, 0, 24, 163, 235, 0, 24, 163, 235, 0,
];
const CYAN_DATA: [u8; 32] = [
    156, 186, 18, 0, 156, 186, 18, 0, 156, 186, 18, 0, 156, 186, 18, 0, 156, 186, 18, 0, 156, 186,
    18, 0, 156, 186, 18, 0, 156, 186, 18, 0,
];
const RED_DATA: [u8; 32] = [
    49, 22, 220, 0, 49, 22, 220, 0, 49, 22, 220, 0, 49, 22, 220, 0, 49, 22, 220, 0, 49, 22, 220, 0,
    49, 22, 220, 0, 49, 22, 220, 0,
];
const PINK_DATA: [u8; 32] = [
    184, 22, 251, 0, 184, 22, 251, 0, 184, 22, 251, 0, 184, 22, 251, 0, 184, 22, 251, 0, 184, 22,
    251, 0, 184, 22, 251, 0, 184, 22, 251, 0,
];
const BLUE_DATA: [u8; 32] = [
    130, 57, 32, 0, 130, 57, 32, 0, 130, 57, 32, 0, 130, 57, 32, 0, 130, 57, 32, 0, 130, 57, 32, 0,
    130, 57, 32, 0, 130, 57, 32, 0,
];

const PHAGE_SILVER_DATA: [u8; 32] = [
    255, 255, 228, 0, 255, 255, 228, 0, 255, 255, 229, 0, 255, 255, 229, 0, 255, 255, 229, 0, 255,
    255, 229, 0, 255, 255, 228, 0, 255, 255, 228, 0,
];
const PHAGE_PINK_DATA: [u8; 32] = [
    148, 8, 221, 0, 148, 8, 221, 0, 149, 4, 222, 0, 150, 0, 224, 0, 150, 0, 224, 0, 150, 0, 224, 0,
    150, 0, 224, 0, 150, 0, 224, 0,
];

const MATCH_OUTLINE_DATA: [u8; 8] = [255, 255, 255, 0, 255, 255, 255, 0];

fn screenshot_game<Conn: ?Sized + RequestConnection>(conn: &Conn, window: Window) -> Vec<u8> {
    conn.get_image(
        ImageFormat::Z_PIXMAP,
        window,
        BOARD_X_OFFSET as i16,
        BOARD_Y_OFFSET as i16,
        BOARD_PIXEL_WIDTH as u16,
        BOARD_PIXEL_HEIGHT as u16,
        !0,
    )
    .unwrap()
    .reply()
    .unwrap()
    .data
}

fn coord_to_offset(x: usize, y: usize) -> usize {
    BYTES_PER_PIXEL * (BOARD_PIXEL_WIDTH * y + x)
}

fn item_from_data(data: &[u8], x: usize, y: usize) -> Item {
    let offset = coord_to_offset(x, y);
    let match_offset = coord_to_offset(x + PIXEL_MATCH_OFFSET, y);

    let matched = &data[match_offset..match_offset + 8] == &MATCH_OUTLINE_DATA;

    let item = if &data[offset..offset + 32] == &YELLOW_DATA {
        Item::File(Color::Yellow)
    } else if &data[offset..offset + 32] == &CYAN_DATA {
        Item::File(Color::Cyan)
    } else if &data[offset..offset + 32] == &RED_DATA {
        Item::File(Color::Red)
    } else if &data[offset..offset + 32] == &PINK_DATA {
        Item::File(Color::Pink)
    } else if &data[offset..offset + 32] == &BLUE_DATA {
        Item::File(Color::Blue)
    } else if &data[offset..offset + 4] == &YELLOW_BOMB_PIXEL {
        Item::Bomb(Color::Yellow)
    } else if &data[offset..offset + 4] == &CYAN_BOMB_PIXEL {
        Item::Bomb(Color::Cyan)
    } else if &data[offset..offset + 4] == &RED_BOMB_PIXEL {
        Item::Bomb(Color::Red)
    } else if &data[offset..offset + 4] == &PINK_BOMB_PIXEL {
        Item::Bomb(Color::Pink)
    } else if &data[offset..offset + 4] == &BLUE_BOMB_PIXEL {
        Item::Bomb(Color::Blue)
    } else {
        Item::Empty
    };

    if matched {
        item.to_matched()
    } else {
        item
    }
}

fn find_y_offset(data: &[u8]) -> Option<usize> {
    for y in (0..BOARD_PIXEL_HEIGHT_ITEMS).rev() {
        for i in 0..board::MAX_COLS {
            let x = i * ITEM_SIZE + PIXEL_X_OFFSET;
            let item = item_from_data(data, x, y);
            if item != Item::Empty {
                return Some(y % ITEM_SIZE);
            }
        }
    }
    None
}

fn find_phage_col(data: &[u8]) -> Option<usize> {
    for col in 0..board::MAX_COLS {
        let x = col * ITEM_SIZE + PHAGE_SILVER_DATA_X_OFFSET;
        let offset = coord_to_offset(x, PHAGE_SILVER_DATA_Y_OFFSET);
        if &data[offset..offset + 32] == &PHAGE_SILVER_DATA {
            return Some(col);
        }

        let offset = coord_to_offset(x - PHAGE_CROUCH_X_OFFSET, PHAGE_SILVER_DATA_Y_OFFSET);
        if &data[offset..offset + 32] == &PHAGE_SILVER_DATA {
            return Some(col);
        }

        let offset = coord_to_offset(x + PHAGE_CROUCH_X_OFFSET, PHAGE_SILVER_DATA_Y_OFFSET);
        if &data[offset..offset + 32] == &PHAGE_SILVER_DATA {
            return Some(col);
        }
    }
    None
}

fn find_pink(data: &[u8], phage_col: usize) -> bool {
    let x = phage_col * ITEM_SIZE + PHAGE_PINK_DATA_X_OFFSET;
    let offset = coord_to_offset(x, PHAGE_PINK_DATA_Y_OFFSET);
    if &data[offset..offset + 32] == &PHAGE_PINK_DATA {
        return true;
    }

    let offset = coord_to_offset(
        x - PHAGE_CROUCH_X_OFFSET,
        PHAGE_PINK_DATA_Y_OFFSET + PHAGE_CROUCH_Y_OFFSET,
    );
    if &data[offset..offset + 32] == &PHAGE_PINK_DATA {
        return true;
    }

    let offset = coord_to_offset(
        x + PHAGE_CROUCH_X_OFFSET,
        PHAGE_PINK_DATA_Y_OFFSET + PHAGE_CROUCH_Y_OFFSET,
    );
    if &data[offset..offset + 32] == &PHAGE_PINK_DATA {
        return true;
    }

    false
}

fn find_held(data: &[u8], phage_col: usize) -> Option<Item> {
    let held_x = phage_col * ITEM_SIZE + PIXEL_X_OFFSET;
    let held = item_from_data(data, held_x, PHAGE_HELD_Y_OFFSET);

    let found_pink = find_pink(data, phage_col);

    if found_pink == (held != Item::Empty) {
        None
    } else {
        Some(held)
    }
}

pub fn get_board_from_window<Conn: ?Sized + RequestConnection>(
    conn: &Conn,
    window: Window,
) -> Option<Board> {
    let image_data = screenshot_game(conn, window);

    let y_offset = match find_y_offset(&image_data) {
        Some(y) => y,
        None => {
            println!("Could not find board y offset");
            return None;
        }
    };

    let mut items = [[Item::Empty; board::MAX_COLS]; board::MAX_ROWS];
    for col in 0..board::MAX_COLS {
        let x = col * ITEM_SIZE + PIXEL_X_OFFSET;
        for row in 0..board::MAX_ROWS {
            let y = row * ITEM_SIZE + y_offset;
            items[row][col] = item_from_data(&image_data, x, y);
        }
    }

    let phage_col = match find_phage_col(&image_data) {
        Some(col) => col,
        None => {
            println!("Could not find phage column");
            return None;
        }
    };

    let held = match find_held(&image_data, phage_col) {
        Some(h) => h,
        None => {
            println!("Could not read held item");
            return None;
        }
    };

    Some(board::make_board(phage_col, held, items))
}

pub fn get_exapunks_window<Conn: ?Sized + RequestConnection>(
    conn: &Conn,
    window: Window,
) -> Option<Window> {
    let reply = conn
        .get_property(false, window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 64)
        .unwrap()
        .reply()
        .unwrap();

    let wm_name = String::from_utf8_lossy(&reply.value);

    println!("{}", wm_name);

    if wm_name == "EXAPUNKS" {
        return Some(window);
    }

    let children = conn.query_tree(window).unwrap().reply().unwrap().children;

    for child in children {
        match get_exapunks_window(conn, child) {
            Some(win) => return Some(win),
            None => (),
        }
    }

    None
}

pub fn validate_window<Conn: ?Sized + RequestConnection>(
    conn: &Conn,
    setup: &Setup,
    screen: &Screen,
    window: Window,
) {
    let geometry = conn.get_geometry(window).unwrap().reply().unwrap();

    assert_eq!(geometry.width, WINDOW_WIDTH);
    assert_eq!(geometry.height, WINDOW_HEIGHT);

    let image_reply = conn
        .get_image(
            ImageFormat::Z_PIXMAP,
            window,
            0,
            0,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            !0,
        )
        .unwrap()
        .reply()
        .unwrap();

    let visual_id = image_reply.visual;

    let image = Image::get_from_reply(setup, WINDOW_WIDTH, WINDOW_HEIGHT, image_reply).unwrap();

    assert_eq!(image.scanline_pad(), ScanlinePad::Pad32);
    assert_eq!(image.depth(), 24u8);
    assert_eq!(image.bits_per_pixel(), BitsPerPixel::B32);

    let visual_type = screen
        .allowed_depths
        .iter()
        .find(|d| d.depth == image.depth())
        .unwrap()
        .visuals
        .iter()
        .find(|v| v.visual_id == visual_id)
        .unwrap();

    assert_eq!(visual_type.red_mask, 0x00ff0000);
    assert_eq!(visual_type.green_mask, 0x0000ff00);
    assert_eq!(visual_type.blue_mask, 0x000000ff);
}

pub fn activate_window<Conn: ?Sized + RequestConnection>(conn: &Conn, window: Window) {
    conn.set_input_focus(InputFocus::NONE, window, x11rb::CURRENT_TIME)
        .unwrap()
        .check()
        .unwrap();

    let mut config = ConfigureWindowAux::new();
    config.stack_mode = Some(StackMode::ABOVE);
    conn.configure_window(window, &config)
        .unwrap()
        .check()
        .unwrap();

    // thread::sleep(time::Duration::from_millis(50));
}

fn keysym_to_keycode<Conn: ?Sized + RequestConnection>(
    conn: &Conn,
    setup: &Setup,
    keysym: Keysym,
) -> Option<Keycode> {
    let mapping = conn
        .get_keyboard_mapping(setup.min_keycode, setup.max_keycode - setup.min_keycode + 1)
        .unwrap()
        .reply()
        .unwrap();

    let syms_per_code = mapping.keysyms_per_keycode as usize;

    for keycode_idx in 0..mapping.length() as usize / syms_per_code {
        for keysym_idx in 0..syms_per_code {
            if mapping.keysyms[keysym_idx + keycode_idx * syms_per_code] == keysym {
                return Some(setup.min_keycode + keycode_idx as u8);
            }
        }
    }

    None
}

pub fn find_keycodes<Conn: ?Sized + RequestConnection>(conn: &Conn, setup: &Setup) -> [Keycode; 4] {
    let mut codes = [0; 4];

    for (i, sym) in [XK_A, XK_D, XK_K, XK_J].iter().enumerate() {
        match keysym_to_keycode(conn, setup, *sym) {
            Some(code) => codes[i] = code,
            None => panic!("No keycode for keysym: {}", sym),
        }
    }

    codes
}

fn send_key<Conn: ?Sized + RequestConnection>(conn: &Conn, key: Keycode) {
    // opcodes found in xproto.h
    // opcode for key press is 2
    // opcode for key release is 3
    xtest::fake_input(conn, 2, key, x11rb::CURRENT_TIME, x11rb::NONE, 0, 0, 0)
        .unwrap()
        .check()
        .unwrap();

    thread::sleep(KEY_DELAY);

    xtest::fake_input(conn, 3, key, x11rb::CURRENT_TIME, x11rb::NONE, 0, 0, 0)
        .unwrap()
        .check()
        .unwrap();

    thread::sleep(KEY_DELAY);
}

pub fn play_path<Conn: ?Sized + RequestConnection>(
    conn: &Conn,
    codes: &[Keycode],
    path: Vec<Move>,
) {
    for m in path {
        match m {
            Move::Left => send_key(conn, codes[0]),     // a
            Move::Right => send_key(conn, codes[1]),    // d
            Move::Swap => send_key(conn, codes[2]),     // k
            Move::Exchange => send_key(conn, codes[3]), // j
        }
    }
}

// checks that the board is seen twice in a row and is different from the given board
pub fn get_new_board<Conn: ?Sized + RequestConnection>(
    conn: &Conn,
    window: Window,
    prev_board: &Board,
) -> Board {
    // let mut failed = 0;
    loop {
        match get_board_from_window(conn, window) {
            Some(board) => {
                if board != *prev_board {
                    return board;
                }
            }
            None => thread::sleep(RECHECK_WAIT_TIME),
        }
    }
}

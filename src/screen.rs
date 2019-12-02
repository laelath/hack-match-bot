use crate::board;
use crate::board::{Board, Color, Item, Move};
use std::mem::MaybeUninit;
use std::{thread, time};
use x11::xlib::{Display, Window, XImage};
use x11::{keysym, xlib, xtest};

pub const KEY_DELAY_MILLIS: u64 = 17;
pub const KEY_DELAY: time::Duration = time::Duration::from_millis(KEY_DELAY_MILLIS);

const EXAPUNKS_WINDOW_NAME: &str = "EXAPUNKS";
const PIXEL_FUZZ: isize = 3;

const ITEM_SIZE: usize = 60;
const BOARD_PIXEL_WIDTH: usize = board::MAX_COLS * ITEM_SIZE;
const BOARD_PIXEL_HEIGHT: usize = 643;
const BOARD_X_OFFSET: usize = 367;
const BOARD_Y_OFFSET: usize = 126;
const BOARD_PIXEL_HEIGHT_ITEMS: usize = 526;

const PIXEL_X_OFFSET: usize = 21;

const WINDOW_WIDTH: usize = 1600;
const WINDOW_HEIGHT: usize = 900;

const XIMAGE_BYTE_ORDER: i32 = xlib::LSBFirst;
const XIMAGE_BITMAP_UNIT: i32 = 32;
const XIMAGE_BITMAP_BIT_ORDER: i32 = xlib::LSBFirst;
const XIMAGE_BITMAP_PAD: i32 = 32;
const XIMAGE_DEPTH: i32 = 24;
const XIMAGE_BYTES_PER_LINE_FULL_WINDOW: i32 = 6400;
const XIMAGE_BITS_PER_PIXEL: i32 = 32;
const XIMAGE_RED_MASK: u64 = 0xFF0000;
const XIMAGE_GREEN_MASK: u64 = 0xFF00;
const XIMAGE_BLUE_MASK: u64 = 0xFF;

const BYTES_PER_PIXEL: usize = XIMAGE_BITS_PER_PIXEL as usize / 8;

#[derive(Clone, Copy)]
struct Pixel(u8, u8, u8);

fn pixel_compare(Pixel(r1, g1, b1): Pixel, Pixel(r2, g2, b2): Pixel) -> bool {
    let r = isize::abs(r1 as isize - r2 as isize);
    let g = isize::abs(g1 as isize - g2 as isize);
    let b = isize::abs(b1 as isize - b2 as isize);

    r + g + b < PIXEL_FUZZ
}

const YELLOW_PIXEL: Pixel = Pixel(235, 163, 24);
const CYAN_PIXEL: Pixel = Pixel(18, 186, 156);
const RED_PIXEL: Pixel = Pixel(220, 22, 49);
const PINK_PIXEL: Pixel = Pixel(251, 22, 184);
const BLUE_PIXEL: Pixel = Pixel(32, 57, 130);
const YELLOW_BOMB_PIXEL: Pixel = Pixel(29, 27, 8);
const CYAN_BOMB_PIXEL: Pixel = Pixel(3, 39, 45);
const RED_BOMB_PIXEL: Pixel = Pixel(66, 9, 15);
const PINK_BOMB_PIXEL: Pixel = Pixel(59, 2, 50);
const BLUE_BOMB_PIXEL: Pixel = Pixel(9, 5, 51);

const PHAGE_HELD_Y_OFFSET: usize = 630;
const PHAGE_PINK_DATA_X_OFFSET: usize = 392 - BOARD_X_OFFSET;
const PHAGE_PINK_DATA_Y_OFFSET: usize = 741 - BOARD_Y_OFFSET;
const PHAGE_SILVER_DATA_X_OFFSET: usize = 385 - BOARD_X_OFFSET;
const PHAGE_SILVER_DATA_Y_OFFSET: usize = 694 - BOARD_Y_OFFSET;

const YELLOW_DATA: [u8; 40] = [
    24, 163, 235, 0, 24, 163, 235, 0, 24, 163, 235, 0, 24, 163, 235, 0, 24, 163, 235, 0, 24, 163,
    235, 0, 24, 163, 235, 0, 24, 163, 235, 0, 24, 163, 235, 0, 24, 163, 235, 0,
];
const CYAN_DATA: [u8; 40] = [
    156, 186, 18, 0, 156, 186, 18, 0, 156, 186, 18, 0, 156, 186, 18, 0, 156, 186, 18, 0, 156, 186,
    18, 0, 156, 186, 18, 0, 156, 186, 18, 0, 156, 186, 18, 0, 156, 186, 18, 0,
];
const RED_DATA: [u8; 40] = [
    49, 22, 220, 0, 49, 22, 220, 0, 49, 22, 220, 0, 49, 22, 220, 0, 49, 22, 220, 0, 49, 22, 220, 0,
    49, 22, 220, 0, 49, 22, 220, 0, 49, 22, 220, 0, 49, 22, 220, 0,
];
const PINK_DATA: [u8; 40] = [
    184, 22, 251, 0, 184, 22, 251, 0, 184, 22, 251, 0, 184, 22, 251, 0, 184, 22, 251, 0, 184, 22,
    251, 0, 184, 22, 251, 0, 184, 22, 251, 0, 184, 22, 251, 0, 184, 22, 251, 0,
];
const BLUE_DATA: [u8; 40] = [
    130, 57, 32, 0, 130, 57, 32, 0, 130, 57, 32, 0, 130, 57, 32, 0, 130, 57, 32, 0, 130, 57, 32, 0,
    130, 57, 32, 0, 130, 57, 32, 0, 130, 57, 32, 0, 130, 57, 32, 0,
];

const PHAGE_SILVER_DATA: [u8; 24] = [
    255, 255, 228, 0, 255, 255, 228, 0, 255, 255, 229, 0, 255, 255, 229, 0, 255, 255, 229, 0, 255,
    255, 228, 0,
];
const PHAGE_PINK_DATA: [u8; 32] = [
    122, 14, 178, 0, 148, 8, 221, 0, 149, 4, 222, 0, 150, 0, 224, 0, 150, 0, 224, 0, 150, 0, 224,
    0, 150, 0, 224, 0, 149, 4, 222, 0,
];

// TODO: check for pieces that are currently in a match
// (right now only finds them if they're under an item)

fn screenshot_game(display: *mut Display, window: Window) -> *mut XImage {
    let img_ptr = unsafe {
        xlib::XGetImage(
            display,
            window,
            BOARD_X_OFFSET as i32,
            BOARD_Y_OFFSET as i32,
            BOARD_PIXEL_WIDTH as u32,
            BOARD_PIXEL_HEIGHT as u32,
            xlib::XAllPlanes(),
            xlib::ZPixmap,
        )
    };
    assert!(!img_ptr.is_null(), "Failed to get window image.");
    img_ptr
}

fn coord_to_offset(x: usize, y: usize) -> usize {
    BYTES_PER_PIXEL * (BOARD_PIXEL_WIDTH * y + x)
}

fn image_compare(d1: &[u8], d2: &[u8]) -> bool {
    let len = usize::min(d1.len(), d2.len()) / BYTES_PER_PIXEL;
    for i in 0..len {
        let x = BYTES_PER_PIXEL * i;
        let p1 = Pixel(d1[x + 2], d1[x + 1], d1[x]);
        let p2 = Pixel(d2[x + 2], d2[x + 1], d2[x]);
        if !pixel_compare(p1, p2) {
            return false;
        }
    }
    true
}

fn item_from_data(data: &[u8], offset: usize) -> Item {
    let pixel = Pixel(data[offset + 2], data[offset + 1], data[offset]);

    if pixel_compare(pixel, YELLOW_PIXEL) {
        if image_compare(&data[offset..], &YELLOW_DATA) {
            return Item::File(Color::Yellow);
        }
    } else if pixel_compare(pixel, CYAN_PIXEL) {
        if image_compare(&data[offset..], &CYAN_DATA) {
            return Item::File(Color::Cyan);
        }
    } else if pixel_compare(pixel, RED_PIXEL) {
        if image_compare(&data[offset..], &RED_DATA) {
            return Item::File(Color::Red);
        }
    } else if pixel_compare(pixel, PINK_PIXEL) {
        if image_compare(&data[offset..], &PINK_DATA) {
            return Item::File(Color::Pink);
        }
    } else if pixel_compare(pixel, BLUE_PIXEL) {
        if image_compare(&data[offset..], &BLUE_DATA) {
            return Item::File(Color::Blue);
        }
    } else if pixel_compare(pixel, YELLOW_BOMB_PIXEL) {
        return Item::Bomb(Color::Yellow);
    } else if pixel_compare(pixel, CYAN_BOMB_PIXEL) {
        return Item::Bomb(Color::Cyan);
    } else if pixel_compare(pixel, RED_BOMB_PIXEL) {
        return Item::Bomb(Color::Red);
    } else if pixel_compare(pixel, PINK_BOMB_PIXEL) {
        return Item::Bomb(Color::Pink);
    } else if pixel_compare(pixel, BLUE_BOMB_PIXEL) {
        return Item::Bomb(Color::Blue);
    }
    Item::Empty
}

fn find_y_offset(data: &[u8]) -> Option<usize> {
    for y in (0..BOARD_PIXEL_HEIGHT_ITEMS).rev() {
        for i in 0..board::MAX_COLS {
            let x = i * ITEM_SIZE + PIXEL_X_OFFSET;
            let offset = coord_to_offset(x, y);
            let item = item_from_data(data, offset);
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
        if image_compare(&data[offset..], &PHAGE_SILVER_DATA) {
            return Some(col);
        }
    }
    None
}

fn find_held(data: &[u8], phage_col: usize) -> Option<Item> {
    let held_x = phage_col * ITEM_SIZE + PIXEL_X_OFFSET;
    let held_offset = coord_to_offset(held_x, PHAGE_HELD_Y_OFFSET);
    let held = item_from_data(data, held_offset);
    let pink_x = phage_col * ITEM_SIZE + PHAGE_PINK_DATA_X_OFFSET;
    let pink_offset = coord_to_offset(pink_x, PHAGE_PINK_DATA_Y_OFFSET);
    let found_pink = image_compare(&data[pink_offset..], &PHAGE_PINK_DATA);

    if found_pink == (held != Item::Empty) {
        None
    } else {
        Some(held)
    }
}

pub fn get_board_from_window(display: *mut Display, window: Window) -> Option<Board> {
    let img_ptr = screenshot_game(display, window);
    let image = unsafe { &mut *img_ptr };
    let image_data = unsafe {
        std::slice::from_raw_parts(
            image.data as *mut u8,
            BOARD_PIXEL_HEIGHT * BOARD_PIXEL_WIDTH * BYTES_PER_PIXEL,
        )
    };
    defer! {{ unsafe { xlib::XDestroyImage(img_ptr); } }}

    let y_offset = match find_y_offset(image_data) {
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
            let offset = coord_to_offset(x, y);
            items[row][col] = item_from_data(image_data, offset);
            if items[row][col] != Item::Empty {
                for k in (0..row).rev() {
                    if items[k][col] == Item::Empty {
                        items[k][col] = Item::Unknown;
                    }
                }
            }
        }
    }

    let phage_col = match find_phage_col(image_data) {
        Some(col) => col,
        None => {
            println!("Could not find phage column");
            return None;
        }
    };

    let held = match find_held(image_data, phage_col) {
        Some(h) => h,
        None => {
            println!("Could not read held item");
            return None;
        }
    };

    Some(board::make_board(phage_col, held, items))
}

pub fn get_exapunks_window(display: *mut Display) -> Window {
    fn get_win_rec(display: *mut Display, window: Window) -> Option<Window> {
        let mut name_ptr = MaybeUninit::uninit();
        let status = unsafe { xlib::XFetchName(display, window, name_ptr.as_mut_ptr()) };

        if status != 0 {
            let name_ptr = unsafe { name_ptr.assume_init() };
            defer! {{ unsafe { xlib::XFree(name_ptr as *mut std::ffi::c_void); } }}

            match unsafe { std::ffi::CStr::from_ptr(name_ptr) }.to_str() {
                Ok(name) => {
                    if name == EXAPUNKS_WINDOW_NAME {
                        return Some(window);
                    }
                }
                Err(_) => (),
            }
        }

        let mut root = MaybeUninit::uninit();
        let mut parent = MaybeUninit::uninit();
        let mut child_ptr = MaybeUninit::uninit();
        let mut num_child = MaybeUninit::uninit();

        unsafe {
            let status = xlib::XQueryTree(
                display,
                window,
                root.as_mut_ptr(),
                parent.as_mut_ptr(),
                child_ptr.as_mut_ptr(),
                num_child.as_mut_ptr(),
            );
            assert_ne!(status, 0, "Failed to query X tree.");
        }

        let _root = unsafe { root.assume_init() };
        let _parent = unsafe { parent.assume_init() };
        let child_ptr = unsafe { child_ptr.assume_init() };
        let num_child = unsafe { num_child.assume_init() };

        defer! {{ unsafe { xlib::XFree(child_ptr as *mut std::ffi::c_void); } }};

        let children = unsafe { std::slice::from_raw_parts(child_ptr, num_child as usize) };

        for child in children {
            match get_win_rec(display, *child) {
                Some(win) => return Some(win),
                None => (),
            }
        }

        None
    }

    let window = unsafe { xlib::XDefaultRootWindow(display) };
    match get_win_rec(display, window) {
        Some(win) => win,
        None => panic!("Failed to get Exapunks window"),
    }
}

pub fn validate_window(display: *mut Display, window: Window) {
    macro_rules! expect_assert {
        ($s:ident, $p:ident, $v:expr) => {{
            let val = $v;
            let real = $s.$p;
            assert_eq!(
                real, val,
                concat!("Expected ", stringify!($p), " = {}, but found {}"),
                val, real
            );
        }};
    }

    let window_attrs = unsafe {
        let mut attrs_ptr = MaybeUninit::uninit();
        let status = xlib::XGetWindowAttributes(display, window, attrs_ptr.as_mut_ptr());
        assert_ne!(status, 0, "Failed to get window attributes");
        attrs_ptr.assume_init()
    };

    expect_assert!(window_attrs, width, WINDOW_WIDTH as i32);
    expect_assert!(window_attrs, height, WINDOW_HEIGHT as i32);

    let img_ptr = unsafe {
        xlib::XGetImage(
            display,
            window,
            0,
            0,
            WINDOW_WIDTH as u32,
            WINDOW_HEIGHT as u32,
            xlib::XAllPlanes(),
            xlib::ZPixmap,
        )
    };
    assert!(!img_ptr.is_null(), "Failed to get window image");

    let image = unsafe { &mut *img_ptr };
    defer! {{ unsafe { xlib::XDestroyImage(img_ptr); } }}

    expect_assert!(image, byte_order, XIMAGE_BYTE_ORDER);
    expect_assert!(image, bitmap_unit, XIMAGE_BITMAP_UNIT);
    expect_assert!(image, bitmap_bit_order, XIMAGE_BITMAP_BIT_ORDER);
    expect_assert!(image, bitmap_pad, XIMAGE_BITMAP_PAD);
    expect_assert!(image, depth, XIMAGE_DEPTH);
    expect_assert!(image, bytes_per_line, XIMAGE_BYTES_PER_LINE_FULL_WINDOW);
    expect_assert!(image, bits_per_pixel, XIMAGE_BITS_PER_PIXEL);
    expect_assert!(image, red_mask, XIMAGE_RED_MASK);
    expect_assert!(image, green_mask, XIMAGE_GREEN_MASK);
    expect_assert!(image, blue_mask, XIMAGE_BLUE_MASK);

    for y in 0..30 {
        for x in 0..30 {
            let correct_pixel = unsafe { xlib::XGetPixel(image, x, y) };
            let correct = (
                (correct_pixel & XIMAGE_RED_MASK) >> 16,
                (correct_pixel & XIMAGE_GREEN_MASK) >> 8,
                correct_pixel & XIMAGE_BLUE_MASK,
            );

            let data_offset =
                XIMAGE_BYTES_PER_LINE_FULL_WINDOW * y + (XIMAGE_BITS_PER_PIXEL / 8) * x;
            let test_pixel = unsafe { *(image.data.offset(data_offset as isize) as *const u64) };
            let test = (
                (test_pixel & image.red_mask) >> 16,
                (test_pixel & image.green_mask) >> 8,
                test_pixel & image.blue_mask,
            );

            assert_eq!(
                correct, test,
                "Failed pixel check, x: {}, y: {}, correct: {:?}, test: {:?}",
                x, y, correct, test
            );
        }
    }
}

pub fn activate_window(display: *mut Display, window: Window) {
    unsafe {
        xlib::XSetInputFocus(display, window, xlib::RevertToNone, xlib::CurrentTime);
        xlib::XRaiseWindow(display, window);
        xlib::XSync(display, xlib::False);
        thread::sleep(time::Duration::from_millis(50));
    }
}

fn send_key(display: *mut Display, key: xlib::KeySym) {
    unsafe {
        let key_code = xlib::XKeysymToKeycode(display, key);
        xtest::XTestFakeKeyEvent(display, key_code as u32, xlib::True, 0);
        xlib::XSync(display, xlib::False);
        thread::sleep(KEY_DELAY);
        xtest::XTestFakeKeyEvent(display, key_code as u32, xlib::False, 0);
        xlib::XSync(display, xlib::False);
        thread::sleep(KEY_DELAY);
    }
}

pub fn move_left(display: *mut Display) {
    send_key(display, keysym::XK_a as xlib::KeySym);
}

pub fn move_right(display: *mut Display) {
    send_key(display, keysym::XK_d as xlib::KeySym);
}

pub fn swap(display: *mut Display) {
    send_key(display, keysym::XK_k as xlib::KeySym);
}

pub fn exchange(display: *mut Display) {
    send_key(display, keysym::XK_j as xlib::KeySym);
}

pub fn play_path(display: *mut Display, path: Vec<Move>) {
    for m in path {
        match m {
            Move::Left => move_left(display),
            Move::Right => move_right(display),
            Move::Swap => swap(display),
            Move::Exchange => exchange(display),
        }
    }
}

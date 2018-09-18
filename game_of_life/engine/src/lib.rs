#![feature(box_syntax, const_transmute)]

extern crate common;
extern crate wasm_bindgen;

use std::mem;
use std::ptr;
use std::u32;

use common::log;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "./index")]
extern "C" {
    pub fn canvasRender(ptr: *const u8);
}

const BOARD_HEIGHT: usize = 50;
const BOARD_WIDTH: usize = 50;
const CELL_COUNT: usize = BOARD_HEIGHT * BOARD_WIDTH;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Dead = unsafe { mem::transmute::<(u8, u8, u8, u8), u32>((0, 0, 0, 255)) },
    Alive = unsafe { mem::transmute::<(u8, u8, u8, u8), u32>((255, 255, 255, 255)) },
}

impl Cell {
    fn is_alive(&self) -> bool {
        *self == Cell::Alive
    }
}

struct Board(pub Box<[Cell; CELL_COUNT]>);

impl Board {
    pub fn new() -> Self {
        let mut cells = [Cell::Dead; CELL_COUNT];
        cells[0] = Cell::Alive;
        cells[1] = Cell::Alive;
        cells[BOARD_WIDTH] = Cell::Alive;
        cells[BOARD_WIDTH + 1] = Cell::Alive;
        Board(box cells)
    }

    pub fn get(&self, x: isize, y: isize) -> Option<Cell> {
        if x < 0 || y < 0 || x >= BOARD_WIDTH as isize || y >= BOARD_HEIGHT as isize {
            return None;
        }

        let index = (y * BOARD_WIDTH as isize) + x;
        Some(self.0[index as usize])
    }
}

struct State {
    pub cur_buf_1: bool,
    pub buf1: Board,
    pub buf2: Board,
}

impl State {
    pub fn new() -> Self {
        State {
            cur_buf_1: true,
            buf1: Board::new(),
            buf2: Board::new(),
        }
    }
}

static mut STATE: *mut State = ptr::null_mut();

#[inline]
fn state() -> &'static mut State {
    unsafe { mem::transmute(STATE) }
}

/// Called by the JS to initialize the game state before starting the simulation
#[wasm_bindgen]
pub fn init() {
    let state = box State::new();
    let state = Box::into_raw(state);
    unsafe { STATE = state as *mut State };
}

fn render(board: &Board) {
    log("Rendering");
    canvasRender(board.0.as_ptr() as *const u8);
}

#[inline]
fn get_coord(index: usize) -> (isize, isize) {
    let x = index % BOARD_WIDTH;
    let y = (index - x) / BOARD_WIDTH;
    return (x as isize, y as isize);
}

#[inline]
fn get_next_cell_state(last_buf: &Board, index: usize) -> Cell {
    let cur_state: Cell = last_buf.0[index];
    let (x, y) = get_coord(index);

    let neighbor_offets: [(isize, isize); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    let live_neighbor_count = neighbor_offets
        .iter()
        .map(|(x_offset, y_offset)| last_buf.get(x + *x_offset, y + *y_offset))
        .filter(Option::is_some)
        .map(Option::unwrap)
        .filter(Cell::is_alive)
        .count();

    if cur_state == Cell::Alive {
        // Die if underpopulated
        if live_neighbor_count < 2 {
            return Cell::Dead;
        }
        // Live if 2 or 3 neighbors
        else if live_neighbor_count == 2 || live_neighbor_count == 3 {
            return Cell::Alive;
        }
        // Die if more than 3 neighbors
        else {
            return Cell::Dead;
        }
    }

    // Spawn new cell if exactly three neighbors
    if live_neighbor_count == 3 {
        return Cell::Alive;
    }

    // Stay dead as the base case
    return Cell::Dead;
}

#[wasm_bindgen]
pub fn tick() {
    log("Tick in Wasm");
    let state = state();
    let (last_board, target_board): (&Board, &mut Board) = if state.cur_buf_1 {
        (&state.buf1, &mut state.buf2)
    } else {
        (&state.buf2, &mut state.buf1)
    };

    log("Starting board iteration");
    for i in 0..CELL_COUNT {
        let new_val_for_cell = get_next_cell_state(last_board, i);
        target_board.0[i] = new_val_for_cell;
    }
    state.cur_buf_1 = !state.cur_buf_1;

    render(target_board);
}
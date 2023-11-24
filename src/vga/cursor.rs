#![allow(unused)]

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::{self, Port};

const WIDTH: usize = 80;

pub struct TextCursor {
    enabled: bool,
    shape: u8,
    row: u8,
    col: u8,
}

#[cfg(target_arch = "x86_64")]
impl TextCursor {
    pub fn new() -> Self {
        TextCursor {
            // Enabled by grub by default
            enabled: true,
            shape: 0,
            row: 0,
            col: 0,
        }
    }

    #[inline]
    fn update_shape(&self) {
        // 0x0A = cursor shape register
        // bit 5 of cursor shape register (13 of this word) controls whether the cursor is enabled
        // bits 0 - 4 of cursor shape register (8 - 12) control the shape
        unsafe {
            Port::new(0x3D4)
                .write(0x00_0A | (!self.enabled as u16) << 13 | (self.shape as u16 & 0x1f) << 8);
        }
    }

    #[inline]
    fn update_position(&self) {
        unsafe {
            let pos: u16 = self.row as u16 * WIDTH as u16 + self.col as u16;
            let mut port = Port::new(0x3D4);
            port.write(0x000F | (pos & 0xFF) << 8);
            port.write(0x000E | ((pos >> 8) & 0xFF) << 8);
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.update_shape()
    }

    pub fn set_shape(&mut self, shape: u8) {
        self.shape = shape;
        self.update_shape()
    }

    pub fn set_position(&mut self, x: u8, y: u8) {
        self.col = x;
        self.row = y;
        self.update_position()
    }
}

lazy_static! {
    pub(crate) static ref CURSOR: Mutex<TextCursor> = Mutex::new(TextCursor::new());
}

pub fn set_enabled(enabled: bool) {
    CURSOR.lock().set_enabled(enabled)
}

pub fn set_shape(shape: u8) {
    CURSOR.lock().set_shape(shape)
}

pub fn set_position(col: u8, row: u8) {
    CURSOR.lock().set_position(col, row)
}

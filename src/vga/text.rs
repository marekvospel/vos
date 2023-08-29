use core::fmt::{self, Write};
use lazy_static::lazy_static;
use spin::Mutex;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    White = 0xf,
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    #[inline]
    pub fn new(foreground: Color, background: Color) -> Self {
        ColorCode(foreground as u8 | (background as u8) << 4_u8)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ColoredChar {
    char: u8,
    color: ColorCode,
}

impl ColoredChar {
    #[inline]
    pub fn new(char: char, color: ColorCode) -> Self {
        ColoredChar {
            char: char as u8,
            color,
        }
    }
}

const WIDTH: usize = 80;
const HEIGHT: usize = 25;

#[repr(transparent)]
pub struct TextBuffer {
    value: [[ColoredChar; WIDTH]; HEIGHT],
}

pub struct TextWriter {
    buffer: &'static mut TextBuffer,
    color: ColorCode,
    row: usize,
    col: usize,
}

impl TextWriter {
    pub fn new() -> Self {
        TextWriter {
            buffer: unsafe { &mut *(0xb8000 as *mut TextBuffer) },
            color: ColorCode::new(Color::White, Color::Black),
            row: 0,
            col: 0,
        }
    }
}

impl TextWriter {
    pub fn clear_line(&mut self, row: usize) {
        self.buffer.value[row] = [ColoredChar {
            char: b' ',
            color: self.color,
        }; 80];
    }

    pub fn new_line(&mut self) {
        self.row += 1;
        self.col = 0;

        if self.row == HEIGHT {
            for row in 0..(HEIGHT - 1) {
                self.buffer.value[row] = self.buffer.value[row + 1];
            }
            self.clear_line(HEIGHT - 1);
            self.row -= 1;
        }
    }
}

impl Write for TextWriter {
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        let c = match c as u8 {
            0x20..=0x7e | b'\n' => c,
            _ => 0xfe as char,
        };

        // new_line
        if c == '\n' {
            self.new_line();
            return Ok(());
        }

        if self.col == WIDTH {
            self.new_line();
        }

        let char = ColoredChar::new(c, self.color);

        self.buffer.value[self.row][self.col] = char;
        self.col += 1;

        Ok(())
    }

    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.chars()
            .map(|c| self.write_char(c))
            .collect::<core::fmt::Result>()
    }
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::text::_print(format_args!($($arg)*)));
}

lazy_static! {
    pub(crate) static ref WRITER: Mutex<TextWriter> = Mutex::new(TextWriter::new());
}

pub fn clear_screen() {
    let mut writer = WRITER.lock();

    for line in 0..HEIGHT {
        writer.clear_line(line)
    }

    writer.row = 0;
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap()
}

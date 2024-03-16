use core::fmt::Write;

use alloc::{string::String, vec::Vec};
use lazy_static::lazy_static;
use spin::Mutex;

pub mod monocraft;

const CHAR_WIDTH: usize = 7;
const CHAR_HEIGHT: usize = 9;

lazy_static! {
    pub static ref LOGGER: Mutex<Option<FramebufferLogger>> = Mutex::new(None);
}

pub(crate) unsafe fn _init(buffer: &multiboot2::FramebufferTag) {
    let mut logger = LOGGER.lock();
    *logger = Some(FramebufferLogger::new(FramebufferInfo::from(buffer)));
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    x: usize,
    y: usize,
}

struct FramebufferInfo {
    address: usize,
    pitch: u32,
    bpp: u8,
    height: u32,
    #[allow(dead_code)]
    width: u32,
}

impl From<&multiboot2::FramebufferTag> for FramebufferInfo {
    fn from(value: &multiboot2::FramebufferTag) -> Self {
        Self {
            address: value.address() as usize,
            pitch: value.pitch(),
            bpp: value.bpp(),
            height: value.height(),
            width: value.width(),
        }
    }
}

pub struct FramebufferLogger {
    framebuffer: FramebufferInfo,
    cursor: Point,
    font_size: usize,
    color: Pixel,
    escape: AnsiEscape,
}

#[derive(Default)]
pub struct AnsiEscape {
    escape: AnsiEscapeParseState,
    args: Vec<u8>,
}

#[derive(Default, Clone)]
pub enum AnsiEscapeParseState {
    #[default]
    None,
    Esc,
    Bracket,
    Argument(String),
}

#[allow(unused)]
pub struct Pixel {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

impl FramebufferLogger {
    pub unsafe fn new(framebuffer: FramebufferInfo) -> FramebufferLogger {
        Self {
            framebuffer,
            cursor: Point { x: 0, y: 0 },
            color: Pixel {
                r: 255,
                g: 255,
                b: 255,
            },
            font_size: 2,
            escape: AnsiEscape::default(),
        }
    }

    pub fn line_width(&self) -> usize {
        self.framebuffer.pitch as usize
            / (self.framebuffer.bpp as usize / 8)
            / ((CHAR_WIDTH + 1) * self.font_size)
    }
    pub fn term_height(&self) -> usize {
        self.framebuffer.height as usize / ((1 + CHAR_HEIGHT) * self.font_size)
    }

    fn next_line(&self) {
        // let term_height = self.term_height();
        // TODO: implement next line function
    }

    fn calculate_offset(&self, (offset_x, offset_y): (usize, usize)) -> (usize, usize) {
        let line_width = self.line_width();
        let offset_y = self.font_size * (1 + CHAR_HEIGHT) * (offset_y + (offset_x / line_width));
        let offset_x = offset_x % line_width;

        (offset_x, offset_y)
    }

    fn character_addr(&self, (offset_x, offset_y): (usize, usize)) -> usize {
        self.framebuffer.address as usize
            + (offset_y as usize * self.framebuffer.pitch as usize)
            + (offset_x as usize
                * self.font_size
                * (CHAR_WIDTH + 1) as usize
                * self.framebuffer.bpp as usize
                / 8)
    }

    pub(crate) fn write_bitmap(&self, offset: (usize, usize), bitmap: u64) {
        let offset = self.calculate_offset(offset);

        let character_addr = self.character_addr(offset);

        for y in 0..CHAR_HEIGHT {
            for x in 0..CHAR_WIDTH {
                let visible = bitmap >> (x + (y * CHAR_WIDTH)) & 1 != 0;

                for yy in 0..self.font_size {
                    for xx in 0..self.font_size {
                        let addr = unsafe {
                            &mut *((character_addr
                                + (((self.font_size * x) + xx) * self.framebuffer.bpp as usize / 8)
                                + ((self.font_size * y) + yy) * self.framebuffer.pitch as usize)
                                as *mut Pixel)
                        };
                        *addr = Pixel {
                            r: visible as u8 * self.color.r,
                            g: visible as u8 * self.color.g,
                            b: visible as u8 * self.color.b,
                        };
                    }
                }
            }
        }
    }

    pub fn write(&mut self, letter: char, font: &dyn Fn(char) -> u64) {
        let line_width = self.line_width();
        let term_height = self.term_height();

        let (mut x, mut y) = self.cursor.into();
        if letter == '\n' {
            x = 0;
            y += 1;
        } else if letter == '\x1b' {
            if matches!(self.escape.escape, AnsiEscapeParseState::None) {
                self.escape.escape = AnsiEscapeParseState::Esc;
            }
            // else just skip adding this invisible letter
        } else if letter == '[' && matches!(self.escape.escape, AnsiEscapeParseState::Esc) {
            self.escape.escape = AnsiEscapeParseState::Bracket;
        } else if matches!(
            self.escape.escape,
            AnsiEscapeParseState::Bracket | AnsiEscapeParseState::Argument(_)
        ) {
            if matches!(self.escape.escape, AnsiEscapeParseState::Bracket) {
                self.escape.escape = AnsiEscapeParseState::Argument(String::with_capacity(2));
            }

            fn reset(me: &mut FramebufferLogger) {
                me.escape = AnsiEscape::default();
            }

            if matches!(letter, '0'..='9') {
                if let AnsiEscapeParseState::Argument(arg) = &mut self.escape.escape {
                    arg.push(letter);
                }
            } else if letter == ';' {
                // TODO: multiple arguments
                if let AnsiEscapeParseState::Argument(arg) = self.escape.escape.clone() {
                    if let Ok(num) = arg.parse() {
                        self.escape.args.push(num);
                    }
                    self.escape.escape = AnsiEscapeParseState::Argument(String::new());
                }
            } else if letter == 'm' {
                // Graphics mode
                if let AnsiEscapeParseState::Argument(arg) = self.escape.escape.clone() {
                    if let Ok(num) = arg.parse() {
                        self.escape.args.push(num);
                    }

                    let mut command_type: i16 = -1;
                    let mut color_type = -1;
                    let (mut r, mut g) = (0, 0);
                    let mut last_arg = 0;

                    for (i, arg) in self.escape.args.clone().into_iter().enumerate() {
                        if command_type != -1 {
                            match command_type {
                                38 => {
                                    if last_arg + 1 == i {
                                        color_type = arg as i8
                                    } else if last_arg + 2 == i {
                                        // TODO other color modes
                                        if color_type == 2 {
                                            r = arg;
                                        }
                                    } else if last_arg + 3 == i && color_type == 2 {
                                        g = arg;
                                    } else if last_arg + 4 == i && color_type == 2 {
                                        self.set_color(Pixel { r, g, b: arg });
                                        command_type = -1;
                                        color_type = -1;
                                        r = 0;
                                        g = 0;
                                        last_arg = i;
                                    }
                                }
                                _ => {}
                            }
                            continue;
                        }

                        last_arg = i + 1;
                        match arg {
                            0 => self.set_color(Pixel {
                                r: 255,
                                g: 255,
                                b: 255,
                            }),
                            31 => self.set_color(Pixel {
                                b: 25,
                                g: 25,
                                r: 200,
                            }),
                            38 => {
                                last_arg -= 1;
                                command_type = 38;
                            }
                            90 => self.set_color(Pixel {
                                r: 150,
                                g: 150,
                                b: 150,
                            }),
                            92 => self.set_color(Pixel {
                                r: 100,
                                g: 244,
                                b: 85,
                            }),
                            _ => {}
                        }
                    }
                }

                reset(self)
            } else {
                todo!();
                reset(self)
            }
        } else {
            let bitmap = font(letter);
            self.write_bitmap(self.cursor.into(), bitmap);

            x += 1;

            if x >= line_width {
                y += 1;
                x = 0;
            }
        }

        if y >= term_height {
            self.next_line();
            y -= 1;
        }

        self.cursor = Point::from((x, y));
    }

    pub fn write_char(&mut self, letter: char) {
        self.write(letter, &monocraft::resolve_letter)
    }

    pub fn write_str(&mut self, str: &str) {
        str.chars().for_each(|c| self.write_char(c))
    }

    pub fn set_color(&mut self, color: Pixel) {
        self.color = color
    }
}

impl Into<(usize, usize)> for Point {
    fn into(self) -> (usize, usize) {
        (self.x, self.y)
    }
}

impl From<(usize, usize)> for Point {
    fn from(value: (usize, usize)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl Write for FramebufferLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Ok(self.write_str(s))
    }
}

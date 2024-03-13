use crate::println;

pub mod monocraft;

const CHAR_WIDTH: usize = 7;
const CHAR_HEIGHT: usize = 9;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    x: usize,
    y: usize,
}

pub struct FramebufferLogger<'a> {
    framebuffer: &'a multiboot2::FramebufferTag,
    cursor: Point,
    font_size: usize,
    color: Pixel,
}

#[allow(unused)]
pub struct Pixel {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

impl<'a> FramebufferLogger<'a> {
    pub unsafe fn new(framebuffer: &'a multiboot2::FramebufferTag) -> FramebufferLogger<'a> {
        Self {
            framebuffer,
            cursor: Point { x: 0, y: 0 },
            color: Pixel {
                r: 255,
                g: 255,
                b: 255,
            },
            font_size: 4,
        }
    }

    pub fn line_width(&self) -> usize {
        self.framebuffer.pitch() as usize
            / (self.framebuffer.bpp() as usize / 8)
            / ((CHAR_WIDTH + 1) * self.font_size)
    }

    fn calculate_offset(&self, (offset_x, offset_y): (usize, usize)) -> (usize, usize) {
        let line_width = self.line_width();
        let offset_y = self.font_size * (1 + CHAR_HEIGHT) * (offset_y + (offset_x / line_width));
        let offset_x = offset_x % line_width;

        (offset_x, offset_y)
    }

    fn character_addr(&self, (offset_x, offset_y): (usize, usize)) -> usize {
        self.framebuffer.address() as usize
            + (offset_y as usize * self.framebuffer.pitch() as usize)
            + (offset_x as usize
                * self.font_size
                * (CHAR_WIDTH + 1) as usize
                * self.framebuffer.bpp() as usize
                / 8)
    }

    pub(crate) fn write_bitmap(&self, offset: (usize, usize), letter: u64) {
        let offset = self.calculate_offset(offset);

        let character_addr = self.character_addr(offset);

        for y in 0..CHAR_HEIGHT {
            for x in 0..CHAR_WIDTH {
                let visible = letter >> (x + (y * CHAR_WIDTH)) & 1 != 0;

                for yy in 0..self.font_size {
                    for xx in 0..self.font_size {
                        let addr = unsafe {
                            &mut *((character_addr
                                + (((self.font_size * x) + xx) * self.framebuffer.bpp() as usize
                                    / 8)
                                + ((self.font_size * y) + yy) * self.framebuffer.pitch() as usize)
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

    fn write(&mut self, letter: u64) {
        // TODO: colors, newlines

        let line_width = self.line_width();
        let term_height = self.framebuffer.height() as usize / (CHAR_HEIGHT * self.font_size);

        self.write_bitmap(self.cursor.into(), letter);

        let (mut x, mut y) = self.cursor.into();
        x += 1;

        if x >= line_width {
            y += 1;
            x = 0;
        }

        if y >= term_height {
            todo!("Reached end of terminal height")
        }
        self.cursor = Point::from((x, y))
    }

    pub fn write_char_font(&mut self, letter: char, font: &dyn Fn(char) -> u64) {
        let result = font(letter);
        self.write(result)
    }

    pub fn write_char(&mut self, letter: char) {
        self.write_char_font(letter, &monocraft::resolve_letter);
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

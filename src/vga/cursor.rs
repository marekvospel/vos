use lazy_static::lazy_static;
use spin::Mutex;

pub struct TextCursor {
    enabled: bool,
    shape: u8,
    row: usize,
    col: usize,
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

    fn update_shape(&self) {
        // 0x0A = cursor shape register
        // bit 5 of cursor shape register (13 of this word) controls whether the cursor is enabled
        // bits 0 - 4 of cursor shape register (8 - 12) control the shape
        unsafe {
            x86::io::outw(
                0x3D4,
                0x00_0A | (!self.enabled as u16) << 13 | (self.shape as u16 & 0x1f) << 8,
            )
        }
    }

    pub fn disable_cursor(&mut self) {
        self.enabled = false;
        self.update_shape()
    }

    pub fn enable_cursor(&mut self) {
        self.enabled = true;
        self.update_shape()
    }

    pub fn set_shape(&mut self, shape: u8) {
        self.shape = shape;
        self.update_shape()
    }
}

lazy_static! {
    pub(crate) static ref CURSOR: Mutex<TextCursor> = Mutex::new(TextCursor::new());
}

pub fn disable_cursor() {
    CURSOR.lock().disable_cursor()
}

pub fn enable_cursor() {
    CURSOR.lock().enable_cursor()
}

pub fn set_shape(shape: u8) {
    CURSOR.lock().set_shape(shape)
}

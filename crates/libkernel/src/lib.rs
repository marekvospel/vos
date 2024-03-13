#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(let_chains)]

extern crate alloc;

use alloc::string::String;
use multiboot2::{BootInformation, BootInformationHeader};

use crate::font::monocraft;

pub mod font;
mod gdt;
mod memory;
mod serial;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    serial_println!("Kernel panic: {info}");
    loop {}
}

#[allow(unused)]
struct Pixel {
    b: u8,
    g: u8,
    r: u8,
}

const CHAR_WIDTH: u64 = 7;
const CHAR_HEIGHT: u64 = 9;

fn write_letter(framebuffer: &multiboot2::FramebufferTag, offset: u64, letter: u64) {
    let char_start =
        framebuffer.address() + offset * (CHAR_WIDTH * 2 + 2) * framebuffer.bpp() as u64 / 8;

    for y in 0..CHAR_HEIGHT {
        for x in 0..CHAR_WIDTH {
            let visible = letter >> (x + (y * CHAR_WIDTH)) & 1 != 0;
            let addr = unsafe {
                &mut *((char_start
                    + (2 * x * framebuffer.bpp() as u64 / 8)
                    + (2 * y) * framebuffer.pitch() as u64) as *mut Pixel)
            };
            *addr = Pixel {
                r: visible as u8 * 255,
                g: visible as u8 * 255,
                b: visible as u8 * 255,
            };

            let addr = unsafe {
                &mut *((char_start
                    + ((2 * x + 1) * framebuffer.bpp() as u64 / 8)
                    + (2 * y) * framebuffer.pitch() as u64) as *mut Pixel)
            };
            *addr = Pixel {
                r: visible as u8 * 255,
                g: visible as u8 * 255,
                b: visible as u8 * 255,
            };
            let addr = unsafe {
                &mut *((char_start
                    + (2 * x * framebuffer.bpp() as u64 / 8)
                    + (2 * y + 1) * framebuffer.pitch() as u64)
                    as *mut Pixel)
            };
            *addr = Pixel {
                r: visible as u8 * 255,
                g: visible as u8 * 255,
                b: visible as u8 * 255,
            };

            let addr = unsafe {
                &mut *((char_start
                    + ((2 * x + 1) * framebuffer.bpp() as u64 / 8)
                    + (2 * y + 1) * framebuffer.pitch() as u64)
                    as *mut Pixel)
            };
            *addr = Pixel {
                r: visible as u8 * 255,
                g: visible as u8 * 255,
                b: visible as u8 * 255,
            };
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_main(multiboot_info_addr: usize) {
    let boot_info =
        unsafe { BootInformation::load(multiboot_info_addr as *const BootInformationHeader) }
            .expect("Error while parsing multiboot header: ");

    let framebuffer = boot_info.framebuffer_tag().unwrap().unwrap();
    println!("{framebuffer:?}");

    println!("Starting VOS...");

    init(&boot_info);

    let str = String::from("Hello world on heap!");
    println!("{}", str);

    let letters = "AB CDEFGHI JKLMNOPQRSTUVWXYZ abcdefghijk";

    for (i, letter) in letters.chars().enumerate() {
        write_letter(&framebuffer, i as u64, monocraft::resolve_letter(letter));
    }

    loop {}
}

fn init(boot_info: &BootInformation) -> () {
    gdt::init_gdt();
    gdt::init_idt();
    memory::init(boot_info);
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::serial::_print(format_args!($($arg)*));
    })
}

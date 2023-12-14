#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(let_chains)]

extern crate alloc;

use alloc::string::String;
use multiboot2::{BootInformation, BootInformationHeader};

mod gdt;
mod memory;
mod serial;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    serial_println!("Kernel panic: {info}");
    loop {}
}

struct Pixel {
    b: u8,
    g: u8,
    r: u8,
}

const LETTER_A: u128 = 0b1_00000000_01011100_01111110_01100110_01110110_01111100_01100000_01110110_00111100_00000000_00000000_00000000;

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

    let char_start = framebuffer.address();
    let char_width = 8;
    let char_height = 12;

    for y in 0..char_height {
        for x in 0..char_width {
            if LETTER_A >> (x + (y * char_width)) & 1 == 0 {
                continue;
            }
            let addr = unsafe {
                &mut *((char_start
                    + (x * framebuffer.bpp() as u64 / 8)
                    + y * framebuffer.pitch() as u64) as *mut Pixel)
            };
            *addr = Pixel {
                r: 255,
                g: 255,
                b: 255,
            };
        }
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

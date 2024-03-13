#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(let_chains)]

extern crate alloc;

use alloc::string::String;
use multiboot2::{BootInformation, BootInformationHeader};

use crate::font::{FramebufferLogger, Pixel};

pub mod font;
mod gdt;
mod memory;
mod serial;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    serial_println!("Kernel panic: {info}");
    loop {}
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

    let mut logger = unsafe { FramebufferLogger::new(framebuffer) };

    logger.set_color(Pixel {
        r: 100,
        g: 100,
        b: 100,
    });
    logger.write_char('[');

    logger.set_color(Pixel { r: 0, g: 255, b: 0 });
    logger.write_char('O');
    logger.write_char('K');

    logger.set_color(Pixel {
        r: 100,
        g: 100,
        b: 100,
    });
    logger.write_char(']');

    logger.set_color(Pixel {
        r: 255,
        g: 255,
        b: 255,
    });
    let letters = " This is my 1st framebuffer message! aaaaaaaaaabbbcccc";
    for letter in letters.chars() {
        logger.write_char(letter);
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

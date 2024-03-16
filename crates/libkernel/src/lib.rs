#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(let_chains)]

extern crate alloc;

use core::{borrow::BorrowMut, ops::DerefMut};

use alloc::string::String;
use multiboot2::{BootInformation, BootInformationHeader};

mod gdt;
mod memory;
mod serial;
pub mod text;

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

    println!("Starting VOS...");

    init(&boot_info);

    let str = String::from("Hello world on heap!");
    println!("{}", str);

    println!(
        "\x1b[90m[\x1b[92mOK\x1b[90m] \x1b[0mThis is my 1st framebuffer message! aaaaaaaaaabbbcccc",
    );
    println!("\x1b[38;2;25;180;209mPoznej markovu barvu");

    loop {}
}

fn init(boot_info: &BootInformation) -> () {
    gdt::init_gdt();
    gdt::init_idt();
    memory::init(boot_info);
    let framebuffer = boot_info.framebuffer_tag().unwrap().unwrap();
    unsafe { text::_init(framebuffer) };
}

pub(crate) fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    if let Some(ref mut logger) = *text::LOGGER.lock() {
        logger
            .write_fmt(args)
            .expect("Printing to framebuffer failed");
    }

    serial::SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::_print(format_args!($($arg)*));
    })
}

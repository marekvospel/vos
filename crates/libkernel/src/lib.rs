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
mod vga;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("Kernel panic: {info}");
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_main(multiboot_info_addr: usize) {
    vga::text::clear_screen();
    vga::cursor::set_enabled(false);

    println!("Starting VOS...");

    let boot_info =
        unsafe { BootInformation::load(multiboot_info_addr as *const BootInformationHeader) }
            .expect("Error while parsing multiboot header: ");

    init(&boot_info);

    let str = String::from("Hello world on heap!");
    println!("{}", str);

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
        $crate::vga::text::_print(format_args!($($arg)*));
        $crate::serial::_print(format_args!($($arg)*));
    })
}

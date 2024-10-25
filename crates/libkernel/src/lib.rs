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

#[no_mangle]
pub extern "C" fn rust_main(multiboot_info_addr: usize) {
    let boot_info =
        unsafe { BootInformation::load(multiboot_info_addr as *const BootInformationHeader) }
            .expect("Error while parsing multiboot header: ");

    let framebuffer = boot_info.framebuffer_tag().unwrap().unwrap();
    println!("{framebuffer:?}");

    println!("Starting VOS...");

    init(&boot_info);

    // let ptr = unsafe { &mut *((framebuffer.address()) as *mut u8) };
    // *ptr = 255;

    // println!("{framebuffer:x?}");

    // let addr = unsafe { &mut *((framebuffer.address()) as *mut u8) };
    // *addr = 255;

    // Memory init is temporarily disabled
    // let str = String::from("Hello world on heap!");
    // println!("{}", str);

    loop {}
}

fn init(boot_info: &BootInformation) -> () {
    gdt::init_gdt();
    gdt::init_idt();
    // memory::init(boot_info);
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

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

#[no_mangle]
pub extern "C" fn rust_main(multiboot_info_addr: usize) {
    let boot_info =
        unsafe { BootInformation::load(multiboot_info_addr as *const BootInformationHeader) }
            .expect("Error while parsing multiboot header: ");

    let framebuffer = boot_info.framebuffer_tag().unwrap().unwrap();
    println!("{framebuffer:?}");

    println!("Starting VOS...");

    init(&boot_info);

    for i in 0..=100000 {
        let addr = unsafe {
            &mut *((framebuffer.address() + (i * framebuffer.bpp() as u64 / 8)) as *mut Pixel)
        };
        *addr = Pixel { r: 255, g: 0, b: 0 };
    }

    // let str = String::from("Hello world on heap!");
    // println!("{}", str);

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

#![no_std]
#![no_main]
#![feature(const_mut_refs)]

extern crate alloc;

use multiboot2::{BootInformation, BootInformationHeader};

mod memory;
mod vga;

#[panic_handler]
pub extern "C" fn panic_handler(info: &core::panic::PanicInfo) -> ! {
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

    init(boot_info);

    loop {}
}

fn init(boot_info: BootInformation) -> () {
    memory::init(boot_info);
}

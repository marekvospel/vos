#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
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
    vga::cursor::disable_cursor();

    println!("Starting VOS...");

    let boot_info =
        unsafe { BootInformation::load(multiboot_info_addr as *const BootInformationHeader) }
            .expect("Error while parsing multiboot header: ");

    println!("Memory areas:");
    for area in boot_info.memory_map_tag().unwrap().memory_areas() {
        println!(
            "   start: 0x{:x}, length: 0x{:x}",
            area.start_address(),
            area.size(),
        );
    }

    println!("kernel sections:");
    for section in boot_info.elf_sections().unwrap() {
        println!(
            "    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
            section.start_address(),
            section.size(),
            section.flags()
        );
    }

    loop {
        vga::cursor::set_shape(8);
        vga::cursor::enable_cursor()
    }
}

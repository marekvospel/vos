#![no_std]
#![no_main]
#![feature(let_chains)]

use archlib::{SetupGDT, SetupInterrupts};
use kernel::rust_main;
use multiboot2::{BootInformation, BootInformationHeader};
use x86lib::X86System;

mod memory;

#[unsafe(no_mangle)]
pub extern "C" fn arch_main(multiboot_info_addr: usize) {
    let boot_info =
        unsafe { BootInformation::load(multiboot_info_addr as *const BootInformationHeader) }
            .expect("Error while parsing multiboot header: ");

    let mut system = X86System::new();

    unsafe {
        system.setup_gdt();
        system.setup_interrupts();
    }
    let heap = memory::init(&boot_info);
    system.set_heap(heap);

    rust_main(system);
}

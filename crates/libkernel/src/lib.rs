#![no_std]
#![no_main]
#![feature(let_chains)]

extern crate alloc;

use alloc::string::String;
use allocator::{LinkedAllocatorNode, LinkedListAllocator};
use x86lib::X86System;

pub mod serial;
pub mod vga;

#[global_allocator]
pub(crate) static mut ALLOCATOR: LinkedListAllocator = LinkedListAllocator::new();

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("Kernel panic: {info}");
    loop {}
}

pub fn rust_main(system: X86System) {
    // TODO: move to x64lib
    vga::cursor::set_enabled(false);
    vga::text::clear_screen();

    let heap = system
        .heap_block()
        .expect("No heap provided by bootstrap process");

    let node = unsafe { &mut *(heap.start as *mut LinkedAllocatorNode) };
    *node = LinkedAllocatorNode::new(heap.size as usize);

    unsafe {
        #[allow(static_mut_refs)]
        ALLOCATOR.init(node);
    }

    let str = String::from("Hello world on heap!");
    println!("{}", str);

    loop {}
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::vga::text::_print(format_args!($($arg)*));
        $crate::serial::_print(format_args!($($arg)*));
    })
}

#![no_std]
#![feature(abi_x86_interrupt)]

extern crate archlib;
extern crate lazy_static;
extern crate x86_64;

use archlib::HeapBlock;
use x86_64::structures::idt::InterruptDescriptorTable;

pub struct X86System {
    heap_block: Option<HeapBlock>,
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

impl X86System {
    pub fn new() -> Self {
        Self { heap_block: None }
    }

    pub(crate) fn idt(&mut self) -> *mut InterruptDescriptorTable {
        &raw mut IDT
    }

    pub fn heap_block(&self) -> Option<HeapBlock> {
        self.heap_block
    }

    pub fn set_heap(&mut self, heap: HeapBlock) {
        self.heap_block = Some(heap.clone());
    }
}

mod gdt;

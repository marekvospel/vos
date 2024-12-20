#![no_std]

pub trait SetupInterrupts {
    unsafe fn setup_interrupts(&mut self);
}

pub trait SetupGDT {
    unsafe fn setup_gdt(&mut self);
}

#[derive(Debug, Clone, Copy)]
pub struct HeapBlock {
    pub start: u64,
    pub size: u64,
}

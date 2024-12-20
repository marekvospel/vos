use crate::X86System;
use archlib::{SetupGDT, SetupInterrupts};
use lazy_static::lazy_static;
use x86_64::{
    VirtAddr,
    instructions::tables::load_tss,
    registers::segmentation::{CS, Segment},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
};

pub(crate) mod interrupts;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[0] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            let stack_end = stack_start + STACK_SIZE as u64;

            stack_end
        };

        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_code = gdt.append(Descriptor::kernel_code_segment());
        let tss = gdt.append(Descriptor::tss_segment(&TSS));

        (gdt, Selectors { kernel_code, tss })
    };
}

struct Selectors {
    kernel_code: SegmentSelector,
    tss: SegmentSelector,
}

impl SetupGDT for X86System {
    unsafe fn setup_gdt(&mut self) {
        GDT.0.load();
        unsafe {
            CS::set_reg(GDT.1.kernel_code);
            load_tss(GDT.1.tss);
        }
    }
}

impl SetupInterrupts for X86System {
    unsafe fn setup_interrupts(&mut self) {
        unsafe {
            let idt = &mut *self.idt();
            idt.breakpoint.set_handler_fn(interrupts::breakpoint);
            idt.page_fault.set_handler_fn(interrupts::page_fault);
            idt.double_fault
                .set_handler_fn(interrupts::double_fault)
                .set_stack_index(0);
            idt.load();
        }
    }
}

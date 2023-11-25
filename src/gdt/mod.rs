use lazy_static::lazy_static;
use x86_64::{
    instructions::tables::load_tss,
    registers::segmentation::{Segment, CS},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        idt::InterruptDescriptorTable,
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use crate::println;

mod interrupt;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[0] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;

            stack_end
        };

        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_code = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (gdt, Selectors { kernel_code, tss })
    };
}

struct Selectors {
    kernel_code: SegmentSelector,
    tss: SegmentSelector,
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub(crate) fn init_gdt() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.kernel_code);
        load_tss(GDT.1.tss);
    }
    println!("[OK] GDT loaded!");
}

pub(crate) fn init_idt() {
    unsafe {
        IDT.breakpoint.set_handler_fn(interrupt::breakpoint);
        IDT.double_fault
            .set_handler_fn(interrupt::double_fault)
            .set_stack_index(0);
        IDT.load();
        println!("[OK] IDT loaded!");
    }
}

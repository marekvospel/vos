use multiboot2::BootInformation;

use crate::memory::frames::FrameAllocator;
use crate::memory::paging::PAGE4;
use crate::println;
use core::ops::RangeInclusive;

pub mod allocator;
pub mod frames;
pub mod paging;

pub(super) fn init(boot_info: BootInformation) -> () {
    println!("Memory areas:");
    let memory_areas = boot_info.memory_map_tag().unwrap().memory_areas();
    for area in memory_areas {
        println!(
            "   start: 0x{:x}, length: 0x{:x}, type: {:?}",
            area.start_address(),
            area.size(),
            area.typ(),
        );
    }

    println!("kernel sections:");
    let elf_sections = boot_info.elf_sections().unwrap();
    for section in elf_sections.clone() {
        println!(
            "    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
            section.start_address(),
            section.size(),
            section.flags()
        );
    }
    let kernel: RangeInclusive<usize> = elf_sections
        .clone()
        .map(|s| s.start_address())
        .min()
        .unwrap() as usize
        ..=elf_sections.map(|s| s.end_address()).max().unwrap() as usize;

    let mut _frame_allocator = FrameAllocator::new(memory_areas, kernel);

    unsafe {
        for index in 0..512 {
            let entry = &(&*PAGE4)[index];
            if !entry.is_unused() {
                println!("Entry {index}: 0x{:x}", entry.0)
            }
        }
    }
}

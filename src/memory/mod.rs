use multiboot2::BootInformation;

use crate::memory::frames::BumpAllocator;
use crate::println;
use core::ops::RangeInclusive;

use self::paging::ActivePageTable;

pub mod allocator;
pub mod frames;
pub mod paging;

pub(super) fn init(boot_info: BootInformation) -> () {
    println!("[INFO] Remapping the kernel...");
    let memory_areas = boot_info.memory_map_tag().unwrap().memory_areas();

    let elf_sections = boot_info.elf_sections().unwrap();
    let kernel: RangeInclusive<u64> = elf_sections
        .clone()
        .map(|s| s.start_address())
        .min()
        .unwrap()
        ..=elf_sections.map(|s| s.end_address()).max().unwrap();

    let mut _frame_allocator = BumpAllocator::new(memory_areas, kernel);
    let mut _active_page = unsafe { ActivePageTable::new() };

    println!("[OK] Nothing crashed")
}

use core::ops::RangeInclusive;

use crate::memory::frames::bump_alloc::BumpAllocator;
use crate::memory::frames::{FrameIter, PhysicalFrame, PAGE_SIZE};
use crate::memory::paging::entry::EntryFlags;
use crate::memory::paging::mapper::ActivePageTable;
use crate::memory::paging::Page;
use crate::println;
use alloc::string::String;
use alloc::vec;
use multiboot2::{BootInformation, ElfSectionFlags};
use x86_64::registers::control::{Cr0, Cr0Flags};
use x86_64::registers::model_specific::{Efer, EferFlags};

use self::frames::FrameAlloc;
use self::paging::inactive::InactivePageTable;
use self::paging::temporary::TemporaryPage;
use allocator::{LinkedAllocatorNode, LinkedListAllocator};

pub mod frames;
pub mod paging;

pub type PhysicalAddress = u64;
pub type VirtualAddress = u64;

pub const TABLE_SIZE: usize = 512;

#[global_allocator]
static mut ALLOCATOR: LinkedListAllocator = LinkedListAllocator::new();

pub(super) fn init(boot_info: &BootInformation) -> () {
    enable_write_protect_bit();
    enable_nxe_bit();

    println!("[INFO] Remapping the kernel...");
    let memory_areas = boot_info.memory_map_tag().unwrap().memory_areas();

    let elf_sections = boot_info.elf_sections().unwrap();
    let kernel: RangeInclusive<u64> = elf_sections
        .clone()
        .map(|s| s.start_address())
        .min()
        .unwrap()
        ..=elf_sections.map(|s| s.end_address()).max().unwrap();

    let mut frame_allocator = BumpAllocator::new(memory_areas, kernel);
    let mut active_page = unsafe { ActivePageTable::new() };

    remap_kernel(&mut frame_allocator, &mut active_page, boot_info);

    println!("[OK] Kernel remapped!");
    println!("[INFO] Initializing linked list allocator...");

    let page = Page::containing_address(0xffff_ff00_0000_0000);
    let frame = frame_allocator.allocate_frame().expect("Out of memory");
    active_page.map_to(page, frame, EntryFlags::WRITABLE, &mut frame_allocator);
    let node: &mut LinkedAllocatorNode = unsafe { &mut *(page.start_address() as *mut _) };

    *node = LinkedAllocatorNode::new(PAGE_SIZE as usize);

    unsafe { ALLOCATOR.init(node) };

    println!("[OK] Linked list allocator initialized!");
}

fn remap_kernel<A: FrameAlloc>(
    allocator: &mut A,
    active_table: &mut ActivePageTable,
    boot_info: &BootInformation,
) {
    let mut temp_page = TemporaryPage::new(Page::containing_address(0x1337a110c), allocator);

    let mut new_table = {
        let frame = allocator.allocate_frame().expect("Out of memory");
        InactivePageTable::new(frame, active_table, &mut temp_page)
    };

    active_table.with(&mut new_table, &mut temp_page, |mapper| {
        for section in boot_info.elf_sections().unwrap() {
            if !section.is_allocated() {
                continue;
            }

            println!(
                "Remapping {:}...",
                section.name().unwrap_or("<Invalid section name>")
            );

            assert!(
                section.start_address() % PAGE_SIZE == 0,
                "sections need to be page aligned"
            );

            let mut flags = EntryFlags::PRESENT;

            if section.flags().contains(ElfSectionFlags::WRITABLE) {
                flags.insert(EntryFlags::WRITABLE);
            }
            if !section.flags().contains(ElfSectionFlags::EXECUTABLE) {
                flags.insert(EntryFlags::NOEXECUTE);
            }

            let start = PhysicalFrame::by_addr(section.start_address());
            let end = PhysicalFrame::by_addr(section.end_address());

            for frame in FrameIter::new(start, end) {
                mapper.identity_map(frame, flags, allocator);
            }
        }
    });

    let old_table = active_table.switch(new_table);

    let old_page = Page::containing_address(old_table.p4_frame.start_address());
    active_table.unmap(old_page, allocator);
}

fn enable_write_protect_bit() {
    unsafe {
        Cr0::update(|flags| {
            flags.insert(Cr0Flags::WRITE_PROTECT);
        })
    };
}

fn enable_nxe_bit() {
    unsafe {
        #[allow(const_item_mutation)]
        Efer::MSR.write(
            (EferFlags::from_bits_truncate(Efer::MSR.read()) | EferFlags::NO_EXECUTE_ENABLE).bits(),
        );
    }
}

use core::ops::{Deref, DerefMut};

use x86_64::{instructions::tlb, registers::control::Cr3};

use crate::memory::{
    frames::{FrameAlloc, PhysicalFrame},
    PhysicalAddress, VirtualAddress, TABLE_SIZE,
};

use super::{
    entry::EntryFlags,
    inactive::InactivePageTable,
    tables::{PageTable, TableLevel4},
    temporary::TemporaryPage,
    Page,
};

pub struct Mapper {
    p4: *mut PageTable<TableLevel4>,
}

impl Mapper {
    unsafe fn new() -> Self {
        Mapper {
            p4: 0xffffffff_fffff000 as *mut _,
        }
    }

    pub fn p4(&self) -> &PageTable<TableLevel4> {
        unsafe { &*self.p4 }
    }

    pub fn p4_mut(&self) -> &mut PageTable<TableLevel4> {
        unsafe { &mut *self.p4 }
    }

    pub fn map_to<A: FrameAlloc>(
        &mut self,
        page: Page,
        frame: PhysicalFrame,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        let p4 = self.p4_mut();

        let p3 = p4.next_level_create(page.p4_index(), allocator);
        let p2 = p3.next_level_create(page.p3_index(), allocator);
        let p1 = p2.next_level_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index() as usize].is_unused());
        p1[page.p1_index() as usize].set(frame, flags | EntryFlags::PRESENT);
    }

    pub fn identity_map<A: FrameAlloc>(
        &mut self,
        frame: PhysicalFrame,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        let page = Page::containing_address(frame.start_address());
        self.map_to(page, frame, flags, allocator)
    }

    pub fn unmap<A: FrameAlloc>(&mut self, page: Page, allocator: &mut A) {
        assert!(self.translate(page.start_address()).is_some());

        let p1 = self
            .p4_mut()
            .next_level_mut(page.p4_index())
            .and_then(|p3| p3.next_level_mut(page.p3_index()))
            .and_then(|p2| p2.next_level_mut(page.p2_index()))
            .expect("mapping code does not support huge pages");
        let frame = p1[page.p1_index() as usize].pointed_frame().unwrap();
        p1[page.p1_index() as usize].set_unused();
        // TODO free p(1,2,3) table if empty
        allocator.deallocate_frame(frame);
    }

    pub fn translate_page(&self, page: Page) -> Option<PhysicalFrame> {
        let p3 = self.p4().next_level(page.p4_index());

        p3.and_then(|p| p.next_level(page.p3_index()))
            .and_then(|p| p.next_level(page.p2_index()))
            .and_then(|p| p[page.p1_index() as usize].pointed_frame())
            .or_else(|| {
                p3.and_then(|p3| {
                    let entry = &p3[page.p3_index() as usize];

                    if let Some(start_frame) = entry.pointed_frame()
                        && entry.flags().contains(EntryFlags::HUGEPAGE)
                    {
                        assert!(start_frame.number % (TABLE_SIZE as u64 * TABLE_SIZE as u64) == 0);
                        return Some(PhysicalFrame {
                            number: start_frame.number
                                + page.p2_index() * TABLE_SIZE as u64
                                + page.p1_index(),
                        });
                    }

                    if let Some(entry) = p3
                        .next_level(page.p3_index())
                        .map(|level| &level[page.p2_index() as usize])
                    {
                        if let Some(start_frame) = entry.pointed_frame()
                            && entry.flags().contains(EntryFlags::HUGEPAGE)
                        {
                            assert!(start_frame.number % TABLE_SIZE as u64 == 0);
                            return Some(PhysicalFrame {
                                number: start_frame.number + page.p1_index(),
                            });
                        }
                    }

                    None
                })
            })
    }

    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virtual_address % TABLE_SIZE as u64;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| frame.number * TABLE_SIZE as u64 + offset)
    }
}

pub struct ActivePageTable {
    mapper: Mapper,
}

impl ActivePageTable {
    pub unsafe fn new() -> Self {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    pub fn with<F>(
        &mut self,
        table: &mut InactivePageTable,
        temporary_page: &mut TemporaryPage,
        f: F,
    ) where
        F: FnOnce(&mut Mapper),
    {
        {
            let backup = Cr3::read();
            let backup = PhysicalFrame::by_addr(backup.0.start_address().as_u64());

            let p4_table = temporary_page.map_table_frame(backup.clone(), self);

            self.p4_mut()[511].set(
                PhysicalFrame {
                    number: table.p4_frame.number,
                },
                EntryFlags::PRESENT | EntryFlags::WRITABLE,
            );
            tlb::flush_all();

            f(self);

            p4_table[511].set(backup, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            tlb::flush_all();
        }
        temporary_page.unmap(self)
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        let old = Cr3::read();

        let old_table = InactivePageTable {
            p4_frame: PhysicalFrame::by_addr(old.0.start_address().as_u64()),
        };
        unsafe {
            Cr3::write(
                x86_64::structures::paging::PhysFrame::containing_address(x86_64::PhysAddr::new(
                    new_table.p4_frame.start_address(),
                )),
                old.1,
            )
        }

        old_table
    }
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Self::Target {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mapper
    }
}

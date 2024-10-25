use crate::memory::{
    frames::{tiny_alloc::TinyAlloc, FrameAlloc, PhysicalFrame},
    paging::entry::EntryFlags,
    VirtualAddress,
};

use super::{
    mapper::ActivePageTable,
    tables::{PageTable, TableLevel1},
    Page,
};

pub struct TemporaryPage {
    page: Page,
    allocator: TinyAlloc,
}

impl TemporaryPage {
    pub fn new<A: FrameAlloc>(page: Page, allocator: &mut A) -> Self {
        TemporaryPage {
            page,
            allocator: TinyAlloc::new(allocator),
        }
    }

    pub fn map(
        &mut self,
        frame: PhysicalFrame,
        active_table: &mut ActivePageTable,
    ) -> VirtualAddress {
        assert!(
            active_table.translate_page(self.page).is_none(),
            "temporary page is already mapped"
        );

        active_table.map_to(self.page, frame, EntryFlags::WRITABLE, &mut self.allocator);
        self.page.start_address()
    }

    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        active_table.unmap(self.page, &mut self.allocator)
    }

    pub fn map_table_frame(
        &mut self,
        frame: PhysicalFrame,
        active_table: &mut ActivePageTable,
    ) -> &mut PageTable<TableLevel1> {
        unsafe { &mut *(self.map(frame, active_table) as *mut PageTable<TableLevel1>) }
    }
}

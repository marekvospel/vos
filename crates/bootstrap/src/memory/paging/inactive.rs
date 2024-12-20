use crate::memory::frames::PhysicalFrame;

use super::{entry::EntryFlags, mapper::ActivePageTable, temporary::TemporaryPage};

pub struct InactivePageTable {
    pub p4_frame: PhysicalFrame,
}

impl InactivePageTable {
    pub fn new(
        frame: PhysicalFrame,
        active_table: &mut ActivePageTable,
        temp_page: &mut TemporaryPage,
    ) -> Self {
        {
            let table = temp_page.map_table_frame(
                PhysicalFrame {
                    number: frame.number,
                },
                active_table,
            );

            table.zero();

            table[511].set(
                PhysicalFrame {
                    number: frame.number,
                },
                EntryFlags::PRESENT | EntryFlags::WRITABLE,
            );
        }
        temp_page.unmap(active_table);

        InactivePageTable { p4_frame: frame }
    }
}

use bitflags::bitflags;

use crate::memory::frames::PhysicalFrame;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct EntryFlags: u64 {
        const PRESENT =         1 << 0;
        const WRITABLE =        1 << 1;
        const USERACCESSIBLE =  1 << 2;
        const WRITETHROUGH =    1 << 3;
        const NO_CACHE =        1 << 4;
        const ACCESSED =        1 << 5;
        const DIRTY =           1 << 6;
        const HUGEPAGE =        1 << 7;
        const GLOBAL =          1 << 8;
        const NOEXECUTE =       1 << 63;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PageEntry(pub(crate) u64);

impl PageEntry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    pub fn set(&mut self, frame: PhysicalFrame, flags: EntryFlags) {
        assert_eq!(frame.start_address() & !0x000fffff_fffff000, 0);
        self.0 = frame.start_address() | flags.bits()
    }

    pub fn pointed_frame(&self) -> Option<PhysicalFrame> {
        if self.flags().contains(EntryFlags::PRESENT) {
            Some(PhysicalFrame::by_addr(self.0 & 0x000fffff_fffff000))
        } else {
            None
        }
    }
}

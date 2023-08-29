use bitflags::bitflags;
use core::ops::{Index, IndexMut};

bitflags! {
  pub struct EntryFlags: u64 {
    const PRESENT    = 1 << 0;
    const WRITABLE   = 1 << 1;
    const HUGEPAGE   = 1 << 7;
  }
}

#[derive(Debug)]
pub struct PageEntry(pub(crate) u64);

impl PageEntry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }
}

const PAGE_SIZE: usize = 512;

pub struct PageTable {
    entries: [PageEntry; PAGE_SIZE],
}

pub const PAGE4: *mut PageTable = 0xffffffff_fffff000 as *mut _;

impl Index<usize> for PageTable {
    type Output = PageEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

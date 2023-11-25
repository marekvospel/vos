use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::memory::frames::FrameAlloc;

use super::{
    entry::{EntryFlags, PageEntry},
    TABLE_SIZE,
};

pub trait TableLevel {}
pub trait PageHiearchy: TableLevel {
    type Target: TableLevel;
}

pub enum TableLevel1 {}
pub enum TableLevel2 {}
pub enum TableLevel3 {}
pub enum TableLevel4 {}

impl TableLevel for TableLevel1 {}
impl TableLevel for TableLevel2 {}
impl TableLevel for TableLevel3 {}
impl TableLevel for TableLevel4 {}

impl PageHiearchy for TableLevel4 {
    type Target = TableLevel3;
}
impl PageHiearchy for TableLevel3 {
    type Target = TableLevel2;
}
impl PageHiearchy for TableLevel2 {
    type Target = TableLevel1;
}

pub struct PageTable<T: TableLevel> {
    entries: [PageEntry; TABLE_SIZE],
    level: PhantomData<T>,
}

impl<T: TableLevel> PageTable<T> {
    pub fn zero(&mut self) -> &mut Self {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
        self
    }
}

impl<T: TableLevel> Index<usize> for PageTable<T> {
    type Output = PageEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<T: TableLevel> IndexMut<usize> for PageTable<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl<H: PageHiearchy> PageTable<H> {
    fn next_level_addr(&self, index: u64) -> Option<u64> {
        let flags = self[index as usize].flags();
        if flags.contains(EntryFlags::PRESENT) {
            let table_address = self as *const _ as u64;
            Some((table_address << 9) | (index << 12))
        } else {
            None
        }
    }

    pub fn next_level(&self, index: u64) -> Option<&PageTable<H::Target>> {
        self.next_level_addr(index)
            .map(|addr| unsafe { &*(addr as *const _) })
    }

    pub fn next_level_mut(&mut self, index: u64) -> Option<&mut PageTable<H::Target>> {
        self.next_level_addr(index)
            .map(|addr| unsafe { &mut *(addr as *mut _) })
    }

    pub fn next_level_create<A: FrameAlloc>(
        &mut self,
        index: u64,
        allocator: &mut A,
    ) -> &mut PageTable<H::Target> {
        match self.next_level(index) {
            Some(_) => self.next_level_mut(index).unwrap(),
            None => {
                assert!(
                    !self.entries[index as usize]
                        .flags()
                        .contains(EntryFlags::HUGEPAGE),
                    "Cannot map to hugepages"
                );
                let frame = allocator.allocate_frame().expect("Out of memory");
                self.entries[index as usize].set(frame, EntryFlags::PRESENT | EntryFlags::WRITABLE);
                self.next_level_mut(index).unwrap().zero()
            }
        }
    }
}

use crate::memory::frames::PhysicalFrame;
use bitflags::bitflags;
use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use super::frames::FrameAlloc;

pub type PhysicalAddress = u64;
pub type VirtualAddress = u64;

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

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    pub fn set(&mut self, frame: PhysicalFrame, flags: EntryFlags) {
        assert_eq!(frame.start_address() & !0x000fffff_fffff000, 0);
        self.0 = (frame.start_address() & 0x000fffff_fffff000) | flags.bits()
    }

    pub fn pointed_frame(&self) -> Option<PhysicalFrame> {
        if self.flags().contains(EntryFlags::PRESENT) {
            Some(PhysicalFrame::by_addr(self.0 & 0x000fffff_fffff000))
        } else {
            None
        }
    }

    pub fn containing_address(address: VirtualAddress) -> Self {
        // Sign extension
        assert!(
            address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000,
            "invalid address: 0x{:x}",
            address
        );
        PageEntry(address / PAGE_SIZE as u64)
    }

    pub(crate) fn start_address(&self) -> PhysicalAddress {
        self.0 * PAGE_SIZE as u64
    }

    pub(crate) fn p4_index(&self) -> u64 {
        (self.0 >> 27) & 0o777
    }
    pub(crate) fn p3_index(&self) -> u64 {
        (self.0 >> 18) & 0o777
    }
    pub(crate) fn p2_index(&self) -> u64 {
        (self.0 >> 9) & 0o777
    }
    pub(crate) fn p1_index(&self) -> u64 {
        (self.0 >> 0) & 0o777
    }
}

const PAGE_SIZE: usize = 512;

pub struct PageTable<T: TableLevel> {
    entries: [PageEntry; PAGE_SIZE],
    level: PhantomData<T>,
}

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

impl<T: TableLevel> PageTable<T> {
    pub fn zero(&mut self) -> &mut Self {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
        self
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

pub struct ActivePageTable {
    p4: *mut PageTable<TableLevel4>,
}

impl ActivePageTable {
    pub unsafe fn new() -> Self {
        ActivePageTable {
            p4: 0xffffffff_fffff000 as *mut _,
        }
    }

    fn p4(&self) -> &PageTable<TableLevel4> {
        unsafe { &*self.p4 }
    }

    fn p4_mut(&self) -> &mut PageTable<TableLevel4> {
        unsafe { &mut *self.p4 }
    }

    pub fn map_to<A: FrameAlloc>(
        &mut self,
        page: PageEntry,
        frame: PhysicalFrame,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        let p4 = self.p4_mut();

        let p3 = p4.next_level_create(page.p4_index(), allocator);
        let p2 = p3.next_level_create(page.p3_index(), allocator);
        let p1 = p2.next_level_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index() as usize].is_unused());
        p1[page.p1_index() as usize].set(frame, flags | EntryFlags::PRESENT)
    }

    pub fn identity_map<A: FrameAlloc>(
        &mut self,
        frame: PhysicalFrame,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        let page = PageEntry::containing_address(frame.start_address());
        self.map_to(page, frame, flags, allocator)
    }

    pub fn unmap<A: FrameAlloc>(&mut self, page: PageEntry, allocator: &mut A) {
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

    pub fn translate_page(&self, page: PageEntry) -> Option<PhysicalFrame> {
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
                        assert!(start_frame.number % (PAGE_SIZE as u64 * PAGE_SIZE as u64) == 0);
                        return Some(PhysicalFrame {
                            number: start_frame.number
                                + page.p2_index() * PAGE_SIZE as u64
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
                            assert!(start_frame.number % PAGE_SIZE as u64 == 0);
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
        let offset = virtual_address % PAGE_SIZE as u64;
        self.translate_page(PageEntry::containing_address(virtual_address))
            .map(|frame| frame.number * PAGE_SIZE as u64 + offset)
    }
}

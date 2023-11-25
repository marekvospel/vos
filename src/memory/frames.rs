use core::ops::RangeInclusive;

use alloc::borrow::ToOwned;
use multiboot2::{MemoryArea, MemoryAreaType};

use super::paging::PhysicalAddress;

pub const PAGE_SIZE: u64 = 4096;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PhysicalFrame {
    pub(crate) number: u64,
}

impl PhysicalFrame {
    pub fn by_addr(addr: PhysicalAddress) -> Self {
        PhysicalFrame {
            number: addr / PAGE_SIZE,
        }
    }

    pub fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    pub fn end_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE + PAGE_SIZE - 1
    }

    pub fn within(&self, range: RangeInclusive<u64>) -> bool {
        return !((range.start().to_owned() > self.start_address()
            && range.end().to_owned() > self.end_address())
            || (range.start().to_owned() < self.start_address()
                && range.end().to_owned() < self.end_address()));
    }
}

pub trait FrameAlloc {
    fn allocate_frame(&mut self) -> Option<PhysicalFrame>;
    fn deallocate_frame(&mut self, frame: PhysicalFrame);
}

pub struct BumpAllocator<'a> {
    next_frame: PhysicalFrame,
    current_area: Option<MemoryArea>,
    areas: &'a [MemoryArea],
    kernel: RangeInclusive<u64>,
}

impl<'a> BumpAllocator<'a> {
    pub fn new(areas: &'a [MemoryArea], kernel: RangeInclusive<u64>) -> Self {
        let mut allocator = BumpAllocator {
            next_frame: PhysicalFrame::by_addr(0),
            current_area: None,
            areas,
            kernel,
        };
        allocator.next_area();
        allocator
    }

    fn next_area(&mut self) {
        self.current_area = self
            .areas
            .iter()
            .filter(|a| {
                self.next_frame < PhysicalFrame::by_addr(a.end_address())
                    && a.typ() == MemoryAreaType::Available
            })
            .min_by_key(|a| a.start_address())
            .map(|v| v.clone());
    }
}

impl<'a> FrameAlloc for BumpAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<PhysicalFrame> {
        if let Some(area) = self.current_area {
            let current_frame = PhysicalFrame {
                number: self.next_frame.number,
            };

            self.next_frame = PhysicalFrame {
                number: self.next_frame.number + 1,
            };

            // Next frame is within the kernel
            if current_frame.within(self.kernel.clone()) {
                // println!("Kernel");
                self.allocate_frame()
            // Next frame is behind current area
            } else if current_frame.end_address() >= area.end_address() {
                self.next_area();
                self.allocate_frame()
            // Next frame is before current area
            } else if current_frame.start_address() < area.start_address() {
                self.next_frame = PhysicalFrame {
                    number: area.start_address() / PAGE_SIZE,
                };
                self.allocate_frame()
            } else {
                Some(current_frame)
            }
        } else {
            None
        }
    }

    fn deallocate_frame(&mut self, frame: PhysicalFrame) {
        unimplemented!()
    }
}

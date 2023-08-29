use core::ops::RangeInclusive;

use alloc::borrow::ToOwned;
use multiboot2::{MemoryArea, MemoryAreaType};

pub const PAGE_SIZE: usize = 4096;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Frame {
    pub(crate) number: usize,
}

impl Frame {
    pub fn by_addr(addr: usize) -> Self {
        Frame {
            number: addr / PAGE_SIZE,
        }
    }

    pub fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    pub fn end_address(&self) -> usize {
        self.number * PAGE_SIZE + PAGE_SIZE - 1
    }

    pub fn within(&self, range: RangeInclusive<usize>) -> bool {
        return !((range.start().to_owned() > self.start_address()
            && range.end().to_owned() > self.end_address())
            || (range.start().to_owned() < self.start_address()
                && range.end().to_owned() < self.end_address()));
    }
}

pub struct FrameAllocator<'a> {
    next_frame: Frame,
    current_area: Option<MemoryArea>,
    areas: &'a [MemoryArea],
    kernel: RangeInclusive<usize>,
}

impl<'a> FrameAllocator<'a> {
    pub fn new(areas: &'a [MemoryArea], kernel: RangeInclusive<usize>) -> Self {
        let mut allocator = FrameAllocator {
            next_frame: Frame::by_addr(0),
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
                self.next_frame < Frame::by_addr(a.end_address() as usize)
                    && a.typ() == MemoryAreaType::Available
            })
            .min_by_key(|a| a.start_address())
            .map(|v| v.clone());
    }

    pub fn allocate_frame(&mut self) -> Option<Frame> {
        if let Some(area) = self.current_area {
            let current_frame = Frame {
                number: self.next_frame.number,
            };

            self.next_frame = Frame {
                number: self.next_frame.number + 1,
            };

            // Next frame is within the kernel
            if current_frame.within(self.kernel.clone()) {
                // println!("Kernel");
                self.allocate_frame()
            // Next frame is behind current area
            } else if current_frame.end_address() >= area.end_address() as usize {
                self.next_area();
                self.allocate_frame()
            // Next frame is before current area
            } else if current_frame.start_address() < area.start_address() as usize {
                self.next_frame = Frame {
                    number: area.start_address() as usize / PAGE_SIZE,
                };
                self.allocate_frame()
            } else {
                Some(current_frame)
            }
        } else {
            None
        }
    }
}

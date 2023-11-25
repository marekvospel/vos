use core::ops::RangeInclusive;

use alloc::borrow::ToOwned;

use super::PhysicalAddress;

pub mod bump_alloc;
pub mod tiny_alloc;

pub const PAGE_SIZE: u64 = 4096;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
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

pub struct FrameIter {
    start: PhysicalFrame,
    end: PhysicalFrame,
}

impl FrameIter {
    pub fn new(start: PhysicalFrame, end: PhysicalFrame) -> Self {
        FrameIter { start, end }
    }
}

impl Iterator for FrameIter {
    type Item = PhysicalFrame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start.number >= self.end.number {
            return None;
        }

        let frame = self.start.clone();
        self.start.number += 1;
        Some(frame)
    }
}

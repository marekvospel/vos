use crate::memory::TABLE_SIZE;

use super::{frames::PAGE_SIZE, PhysicalAddress, VirtualAddress};

pub mod entry;
pub mod inactive;
pub mod mapper;
pub mod tables;
pub mod temporary;

#[derive(Debug, Clone, Copy)]
pub struct Page {
    pub(crate) number: u64,
}

impl Page {
    pub fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(
            address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000,
            "invalid address: 0x{:x}",
            address
        );
        Page {
            number: address / PAGE_SIZE,
        }
    }

    pub(crate) fn p4_index(&self) -> u64 {
        (self.number >> 27) & 0o777
    }
    pub(crate) fn p3_index(&self) -> u64 {
        (self.number >> 18) & 0o777
    }
    pub(crate) fn p2_index(&self) -> u64 {
        (self.number >> 9) & 0o777
    }
    pub(crate) fn p1_index(&self) -> u64 {
        (self.number >> 0) & 0o777
    }
}

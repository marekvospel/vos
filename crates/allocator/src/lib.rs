#![cfg_attr(not(test), no_std)]

pub use self::allocator::LinkedListAllocator;
pub use self::linked_list::{LinkedAllocatorIter, LinkedAllocatorNode};

pub(crate) mod allocator;
pub(crate) mod linked_list;

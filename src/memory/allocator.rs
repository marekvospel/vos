use core::alloc::{GlobalAlloc, Layout};

#[global_allocator]
static ALLOCATOR: MemAllocator = MemAllocator {};

pub struct MemAllocator {}

unsafe impl<'a> GlobalAlloc for MemAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        unimplemented!("Allocate")
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        unimplemented!("Deallocate")
    }
}

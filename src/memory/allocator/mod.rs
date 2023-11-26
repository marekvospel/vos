use core::{
    alloc::{GlobalAlloc, Layout},
    mem::transmute,
};

static mut NODES: Option<&mut LinkedAllocatorNode> = None;

#[global_allocator]
static ALLOCATOR: LinkedListAllocator = LinkedListAllocator {};

pub struct LinkedListAllocator {}

unsafe impl<'a> GlobalAlloc for LinkedListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if NODES.is_none() {
            panic!("LinkedListAllocator has not been initialized yet");
        }

        let node = NODES.as_deref_mut().unwrap();

        if layout.size() > node.size {
            panic!("Out of heap memory");
        }

        node.size = node.size - layout.size();
        let ptr = node as *mut _ as usize + node.size;

        ptr as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // unimplemented!("Deallocate")
    }
}

pub unsafe fn init(node: &'static mut LinkedAllocatorNode) {
    if NODES.is_some() {
        panic!("LinkedListAllocator is already initialized");
    }
    NODES = Some(node);
}

pub struct LinkedAllocatorNode<'a> {
    pub(crate) size: usize,
    pub(crate) next: Option<&'a mut Self>,
}

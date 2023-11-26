use core::{
    alloc::{GlobalAlloc, Layout},
    mem::size_of,
};

use self::linked_list::LinkedAllocatorNode;

pub mod linked_list;

static mut NODES: Option<&mut LinkedAllocatorNode> = None;

#[global_allocator]
static ALLOCATOR: LinkedListAllocator = LinkedListAllocator {};

pub struct LinkedListAllocator {}

unsafe impl<'a> GlobalAlloc for LinkedListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if NODES.is_none() {
            panic!("LinkedListAllocator has not been initialized yet");
        }

        let start_node = NODES.as_deref_mut().unwrap();

        let mut last_node = &mut *(start_node as *mut LinkedAllocatorNode);
        for node in start_node.as_iter() {
            let aligned_addr =
                (node.end_address() - layout.size()) / layout.align() * layout.align();
            let valid_addr;

            if aligned_addr == node.end_address() - layout.size() {
                // section is at the end, no need to create new LinkedAllocatorNode
                valid_addr = aligned_addr;
            } else {
                // section isn't at the end, add padding
                valid_addr =
                    (node.end_address() - layout.size() - size_of::<LinkedAllocatorNode>())
                        / layout.align()
                        * layout.align();
            }

            if valid_addr < node.start_address() {
                last_node = node;
                continue;
            }

            node.size = valid_addr - node.start_address();

            if aligned_addr != valid_addr {
                let new_node = unsafe {
                    &mut *((node.end_address() - size_of::<LinkedAllocatorNode>())
                        as *mut LinkedAllocatorNode)
                };
                new_node.size = 0;
                new_node.next = node.next;
                last_node.next = Some(new_node);
            }

            return valid_addr as *mut u8;
        }

        panic!("No section in heap matching {:?}", layout)
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

use core::{
    alloc::{GlobalAlloc, Layout},
    cmp::max,
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

        // Min alloc size is size of LinkedListAllocator, so when deallocating, the Node tag can be inserted
        let size = max(layout.size(), size_of::<LinkedAllocatorNode>());
        let start_node = NODES.as_deref_mut().unwrap();

        let mut last_node = &mut *(start_node as *mut LinkedAllocatorNode);
        for node in start_node.as_iter() {
            let aligned_addr = (node.end_address() + 1 - size) / layout.align() * layout.align();
            let valid_addr;

            if aligned_addr == node.end_address() + 1 - size {
                // section is at the end, no need to create new LinkedAllocatorNode
                valid_addr = aligned_addr;
            } else {
                // section isn't at the end, add padding
                valid_addr = (node.end_address() + 1 - size - size_of::<LinkedAllocatorNode>())
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
                    &mut *((node.end_address() + 1 - size_of::<LinkedAllocatorNode>())
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

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if NODES.is_none() {
            panic!("LinkedListAllocator has not been initialized yet");
        }

        // Min alloc size is size of LinkedListAllocator, so when deallocating, the Node tag can be inserted
        let size = max(layout.size(), size_of::<LinkedAllocatorNode>());
        let start_node = NODES.as_deref_mut().unwrap();

        let mut last_node = &mut *(start_node as *mut LinkedAllocatorNode);
        let mut iter = start_node.as_iter();
        while let Some(node) = iter.next() {
            // ptr is right next to an existing node
            if node.end_address() + 1 == ptr as usize {
                last_node.size += if let Some(next) = node.next
                    && node.end_address() + 1 == next as *const _ as usize
                {
                    let next = &mut *next;
                    last_node.next = next.next;
                    size + size_of::<LinkedAllocatorNode>() + next.size
                } else {
                    size
                };

                if let Some(next) = last_node.next
                    && last_node.end_address() + 1 == next as usize
                {
                    let next = &*next;
                    last_node.size += size_of::<LinkedAllocatorNode>() + next.size;
                    last_node.next = next.next;
                }

                return;
            } else if ptr as usize > last_node.end_address() {
                if node.next.is_none() || (ptr as usize) < node as *const _ as usize {
                    let middle_node = &mut *(ptr as *mut LinkedAllocatorNode);
                    middle_node.size = size - size_of::<LinkedAllocatorNode>();
                    middle_node.next =
                        if node as *const _ as usize == last_node as *const _ as usize {
                            node.next
                        } else {
                            Some(node)
                        };
                    last_node.next = Some(middle_node);

                    return;
                }
            }

            last_node = node;
        }

        panic!("Could not deallocate 0x{:x} {:?}", ptr as usize, layout);
    }
}

pub unsafe fn init(node: &'static mut LinkedAllocatorNode) {
    if NODES.is_some() {
        panic!("LinkedListAllocator is already initialized");
    }
    NODES = Some(node);
}

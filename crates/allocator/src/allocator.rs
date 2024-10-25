use core::{
    alloc::{GlobalAlloc, Layout},
    mem::size_of,
};

use crate::LinkedAllocatorNode;

pub struct LinkedListAllocator {
    pub nodes: Option<*mut LinkedAllocatorNode>,
}

unsafe impl Send for LinkedListAllocator {}

unsafe impl<'a> GlobalAlloc for LinkedListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if self.nodes.is_none() {
            panic!("LinkedListAllocator has not been initialized yet");
        }

        let start_node = &mut *(self.nodes_mut().unwrap());

        for node in start_node.as_iter() {
            if !node.free {
                continue;
            }

            // Find where to put the allocation
            let mut aligned_addr = (node.end_address() + 1 - layout.size()) / layout.align()
                * layout.align()
                - size_of::<LinkedAllocatorNode>();
            let mut empty_node = None;

            // section isn't at the end, add padding that is at least LinkedAllocatorNode big
            if aligned_addr
                != node.end_address() + 1 - layout.size() - size_of::<LinkedAllocatorNode>()
            {
                aligned_addr =
                    (node.end_address() + 1 - layout.size() - size_of::<LinkedAllocatorNode>())
                        / layout.align()
                        * layout.align()
                        - size_of::<LinkedAllocatorNode>();

                empty_node = Some(
                    (aligned_addr + layout.size() + size_of::<LinkedAllocatorNode>())
                        as *mut LinkedAllocatorNode,
                );
                let empty_node = &mut *empty_node.unwrap();
                empty_node.free = true;
                empty_node.size = (node.end_address() + 1)
                    - (aligned_addr + layout.size() + 2 * size_of::<LinkedAllocatorNode>());

                assert_eq!(empty_node.end_address(), node.end_address());
                empty_node.next = node.next;
            }

            // If there is space for our allocation and its node
            if aligned_addr < node.start_address() {
                continue;
            }

            node.size = 0;

            let new_node = &mut *((aligned_addr) as *mut LinkedAllocatorNode);
            new_node.size = layout.size();
            new_node.free = false;
            if empty_node.is_some() {
                new_node.next = empty_node;
            } else {
                new_node.next = node.next;
            }

            // TODO: handle this, possibly a rush condition, fuck locks but im too lazy to write
            // actual atomic allocator xd
            if node.size != 0 {
                // println!("! Weird stuff happening not good, mod.rs:54");
            }

            // Successful alloc
            node.next = Some(new_node);
            node.size = aligned_addr - node.start_address();

            return (aligned_addr + size_of::<LinkedAllocatorNode>()) as *mut u8;
        }

        panic!("No section in heap matching {:?}", layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if self.nodes.is_none() {
            panic!("LinkedListAllocator has not been initialized yet");
        }

        // Min alloc size is size of LinkedListAllocator, so when deallocating, the Node tag can be inserted
        let start_node = &mut *(self.nodes_mut().unwrap());

        let mut last_node = &mut *(start_node as *mut LinkedAllocatorNode);
        let mut iter = start_node.as_iter();
        while let Some(node) = iter.next() {
            if ptr as usize == node.start_address() {
                let mut next_node = iter.next();
                node.free = true;

                // Merge with next allocation
                if let Some(ref nnode) = next_node {
                    if nnode.free && node.end_address() + 1 == (*nnode) as *const _ as usize {
                        node.next = nnode.next;
                        node.size += nnode.size + size_of::<LinkedAllocatorNode>();
                        next_node = iter.next();
                    }
                }

                // Merge with previous allocation
                if last_node.free {
                    last_node.size += node.size + size_of::<LinkedAllocatorNode>();
                    last_node.next = next_node.map(|n| n as *mut LinkedAllocatorNode);
                }

                return;
            }

            last_node = node;
        }

        panic!("Could not deallocate 0x{:x} {:?}", ptr as usize, layout);
    }
}

impl LinkedListAllocator {
    pub const fn new() -> LinkedListAllocator {
        LinkedListAllocator { nodes: None }
    }

    pub fn init(&mut self, node: &'static mut LinkedAllocatorNode) {
        if self.nodes.is_some() {
            panic!("LinkedListAllocator is already initialized");
        }
        self.nodes = Some(node as *mut _);
    }

    pub unsafe fn nodes_mut(&self) -> Option<*mut LinkedAllocatorNode> {
        self.nodes.map(|n| n as *mut _)
    }
}

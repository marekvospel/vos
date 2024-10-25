use allocator::*;
use core::alloc::{GlobalAlloc, Layout};

pub unsafe fn print_nodes(alloc: &LinkedListAllocator) {
    if alloc.nodes.is_none() {
        panic!("LinkedListAllocator has not been initialized yet");
    }

    let start_node = &mut *alloc.nodes.unwrap();
    for node in start_node.as_iter() {
        println!("0x{:x}: Node: {node:?}", node as *const _ as usize);
    }
}

fn create_node(ptr: usize, size: usize) -> *mut LinkedAllocatorNode {
    let node: &mut LinkedAllocatorNode = unsafe { std::mem::transmute(ptr) };
    node.next = None;
    node.size = size - size_of::<LinkedAllocatorNode>();
    node.free = true;
    node as *mut _
}

#[test]
fn should_initialize_allocator_once() {
    const SIZE: usize = 256;
    let mut buf = [0u8; SIZE];
    let node = unsafe { &mut *create_node(&mut buf[0] as *const _ as usize, SIZE) };

    let mut allocator = LinkedListAllocator::new();
    allocator.init(node);

    unsafe {
        let ptr = allocator.alloc(Layout::from_size_align_unchecked(32, 1));
        allocator.dealloc(ptr, Layout::from_size_align_unchecked(32, 1));
    }
}

#[test]
fn should_merge_with_previous_node() {
    const SIZE: usize = 256;
    let mut buf = [0u8; SIZE];
    let node = create_node(&mut buf[0] as *const _ as usize, SIZE);

    let init_node = unsafe { &mut *node };
    let node = unsafe { &mut *node };

    let mut allocator = LinkedListAllocator::new();
    allocator.init(init_node);

    assert!(node.free);
    assert_eq!(node.size, 256 - size_of::<LinkedAllocatorNode>());
    assert_eq!(node.next, None);

    unsafe {
        let ptr = allocator.alloc(Layout::from_size_align_unchecked(32, 1));
        allocator.dealloc(ptr, Layout::from_size_align_unchecked(32, 1));
    }

    assert!(node.free);
    assert_eq!(node.size, 256 - size_of::<LinkedAllocatorNode>());
    assert_eq!(node.next, None);
}

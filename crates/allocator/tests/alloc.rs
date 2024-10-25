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
#[should_panic]
fn should_fail_double_initing() {
    const SIZE: usize = 256;
    let mut buf = [0u8; SIZE];
    let mut buf2 = [0u8; SIZE];
    let node = unsafe { &mut *create_node(&mut buf[0] as *const _ as usize, SIZE) };
    let node2 = unsafe { &mut *create_node(&mut buf2[0] as *const _ as usize, SIZE) };

    let mut allocator = LinkedListAllocator::new();
    allocator.init(node);
    allocator.init(node2);
}

#[test]
#[should_panic]
fn should_fail_not_initialized() {
    let allocator = LinkedListAllocator::new();

    unsafe {
        allocator.alloc(Layout::from_size_align_unchecked(32, 1));
    }
}

#[test]
fn should_merge_with_previous_node() {
    const SIZE: usize = 256;
    let mut buf = [0u8; SIZE];
    let node = unsafe { &mut *create_node(&mut buf[0] as *const _ as usize, SIZE) };

    let mut allocator = LinkedListAllocator::new();
    allocator.init(node);

    unsafe {
        let ptr = allocator.alloc(Layout::from_size_align_unchecked(32, 1));
        allocator.dealloc(ptr, Layout::from_size_align_unchecked(32, 1));
    }

    let node = unsafe { &*(allocator.nodes_mut().unwrap()) };
    assert!(node.free);
    assert_eq!(node.size, SIZE - size_of::<LinkedAllocatorNode>());
    assert_eq!(node.next, None);
}

#[test]
fn should_merge_with_next_node() {
    const SIZE: usize = 256;
    let mut buf = [0u8; SIZE];
    let node = create_node(&mut buf[0] as *const _ as usize, SIZE);

    let node = unsafe { &mut *node };

    let mut allocator = LinkedListAllocator::new();
    allocator.init(node);

    unsafe {
        let ptr1 = allocator.alloc(Layout::from_size_align_unchecked(32, 32));
        let ptr2 = allocator.alloc(Layout::from_size_align_unchecked(5, 1));

        print_nodes(&allocator);

        // Start, ptr2, ptr1, reserved
        {
            let node = &*(allocator.nodes_mut().unwrap());
            assert_eq!(node.as_iter().count(), 4);
        }

        allocator.dealloc(ptr1, Layout::from_size_align_unchecked(32, 32));

        print_nodes(&allocator);

        // Start, ptr2, free
        {
            let node = &*(allocator.nodes_mut().unwrap());
            assert_eq!(node.as_iter().count(), 3);
        }

        allocator.dealloc(ptr2, Layout::from_size_align_unchecked(5, 1));
    }

    // Free
    let node = unsafe { &*(allocator.nodes_mut().unwrap()) };

    assert!(node.free);
    assert_eq!(node.size, SIZE - size_of::<LinkedAllocatorNode>());
    assert_eq!(node.next, None);
}

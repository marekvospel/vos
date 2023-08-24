use core::mem::size_of;

#[derive(Debug)]
pub struct LinkedAllocatorNode {
    pub size: usize,
    pub free: bool,
    pub next: Option<*mut Self>,
}

impl LinkedAllocatorNode {
    pub fn new(size: usize) -> Self {
        LinkedAllocatorNode {
            size: size - size_of::<Self>(),
            free: true,
            next: None,
        }
    }

    pub fn start_address(&self) -> usize {
        self as *const _ as usize + size_of::<Self>()
    }

    // Returns the last address in this node
    pub fn end_address(&self) -> usize {
        self.start_address() + (self.size - 1)
    }

    pub fn as_iter<'a>(&'a self) -> LinkedAllocatorIter {
        LinkedAllocatorIter {
            current: Some(self as *const _ as usize as *mut LinkedAllocatorNode),
        }
    }
}

pub struct LinkedAllocatorIter {
    current: Option<*mut LinkedAllocatorNode>,
}

impl Iterator for LinkedAllocatorIter {
    type Item = &'static mut LinkedAllocatorNode;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;

        let deref = unsafe { &mut (*current) };
        self.current = deref.next;

        Some(deref)
    }
}

use super::{FrameAlloc, PhysicalFrame};

pub struct TinyAlloc {
    frames: [Option<PhysicalFrame>; 3],
}

impl TinyAlloc {
    pub fn new<A: FrameAlloc>(allocator: &mut A) -> Self {
        Self {
            frames: [
                allocator.allocate_frame(),
                allocator.allocate_frame(),
                allocator.allocate_frame(),
            ],
        }
    }
}

impl FrameAlloc for TinyAlloc {
    fn allocate_frame(&mut self) -> Option<PhysicalFrame> {
        for frame in &mut self.frames {
            if frame.is_some() {
                return frame.take();
            }
        }
        None
    }

    fn deallocate_frame(&mut self, frame: PhysicalFrame) {
        match self.frames.iter_mut().find(|f| f.is_none()).and_then(|f| {
            *f = Some(frame);
            Some(())
        }) {
            Some(_) => {}
            None => panic!("Tiny allocator already has 3 frames"),
        }
    }
}

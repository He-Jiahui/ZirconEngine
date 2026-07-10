use super::PoseBuffer;

/// Reusable pose-buffer storage for allocation-free steady-state evaluation.
#[derive(Debug)]
pub struct PosePool {
    available: Vec<PoseBuffer>,
    miss_count: u64,
}

impl PosePool {
    pub fn with_buffers(buffer_count: usize, joint_capacity: usize) -> Self {
        let mut available = Vec::with_capacity(buffer_count);
        for _ in 0..buffer_count {
            available.push(PoseBuffer::with_capacity(joint_capacity));
        }
        Self {
            available,
            miss_count: 0,
        }
    }

    pub fn acquire(&mut self, joint_count: usize) -> PoseBuffer {
        let mut buffer = self.available.pop().unwrap_or_else(|| {
            self.miss_count = self.miss_count.saturating_add(1);
            PoseBuffer::with_capacity(joint_count)
        });
        if buffer.joint_capacity() < joint_count {
            self.miss_count = self.miss_count.saturating_add(1);
        }
        buffer.reset(joint_count);
        buffer
    }

    pub fn release(&mut self, mut buffer: PoseBuffer) {
        buffer.clear();
        self.available.push(buffer);
    }

    pub fn miss_count(&self) -> u64 {
        self.miss_count
    }

    pub fn available_count(&self) -> usize {
        self.available.len()
    }
}

//! 剪辑评估器按骨架缓存可复用姿态缓冲区，减少连续帧的分配；取得的缓冲区必须按当前关节数重置后再采样。
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

    /// 暂借按当前关节数重置的缓冲区；使用结束后归还，否则后续帧可能增加分配。
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

    /// 清空逻辑行数并保留容量；池以骨架缓存为所有者，不跨评估器共享。
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

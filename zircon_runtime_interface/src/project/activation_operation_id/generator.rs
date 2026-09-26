use std::sync::atomic::{AtomicU64, Ordering};

use uuid::Uuid;

use super::{
    ProjectActivationOperationId, ProjectActivationOperationSequence, ProjectLaunchInstanceId,
};

// 零值是序号类型拒绝的耗尽哨兵；最后一个 u64 序号只会发放一次。
const EXHAUSTED_SEQUENCE: u64 = 0;

/// 为同一启动进程实例并发分配项目激活标识；Hub、CLI 与 Editor 各自持有进程级生成器。
pub struct ProjectActivationOperationIdGenerator {
    origin_instance: ProjectLaunchInstanceId,
    next_sequence: AtomicU64,
}

impl ProjectActivationOperationIdGenerator {
    pub const fn new(origin_instance: ProjectLaunchInstanceId) -> Self {
        Self {
            origin_instance,
            next_sequence: AtomicU64::new(1),
        }
    }

    pub const fn origin_instance(&self) -> ProjectLaunchInstanceId {
        self.origin_instance
    }

    /// 原子地占用一个非零序号，并以随机 nonce 区分操作；序号耗尽后返回 `None`。
    pub fn allocate(&self) -> Option<ProjectActivationOperationId> {
        loop {
            let current = self.next_sequence.load(Ordering::Relaxed);
            let sequence = ProjectActivationOperationSequence::new(current)?;
            let next = current.checked_add(1).unwrap_or(EXHAUSTED_SEQUENCE);
            if self
                .next_sequence
                .compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return Some(
                    ProjectActivationOperationId::try_from_parts(
                        self.origin_instance,
                        sequence,
                        Uuid::new_v4(),
                    )
                    .expect("a v4 UUID cannot be nil"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocator_emits_the_last_valid_sequence_once_before_exhaustion() {
        let origin_instance = ProjectLaunchInstanceId::new();
        let generator = ProjectActivationOperationIdGenerator {
            origin_instance,
            next_sequence: AtomicU64::new(u64::MAX),
        };

        let final_operation = generator.allocate().expect("last sequence");

        assert_eq!(final_operation.origin_instance(), origin_instance);
        assert_eq!(final_operation.sequence().get(), u64::MAX);
        assert_eq!(generator.allocate(), None);
    }
}

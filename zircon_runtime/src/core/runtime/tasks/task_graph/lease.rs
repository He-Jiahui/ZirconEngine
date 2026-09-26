use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// 只追踪公开 TaskGraphScope 克隆，不把任务句柄或图内部引用误算作继续接纳的所有者。
// 最后一个客户端释放时由 scope 的 Drop 关闭接纳，已接纳任务仍按图生命周期结算。
pub(super) struct TaskGraphClientLease {
    owner_count: AtomicUsize,
}

impl TaskGraphClientLease {
    pub(super) fn new() -> Arc<Self> {
        Arc::new(Self {
            owner_count: AtomicUsize::new(1),
        })
    }

    pub(super) fn retain(&self) {
        let prior_owner_count = self.owner_count.fetch_add(1, Ordering::Relaxed);
        debug_assert!(
            prior_owner_count < usize::MAX,
            "execution client lease overflow"
        );
    }

    pub(super) fn release(&self) -> bool {
        let prior_owner_count = self.owner_count.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(prior_owner_count > 0, "execution client lease underflow");
        prior_owner_count == 1
    }
}

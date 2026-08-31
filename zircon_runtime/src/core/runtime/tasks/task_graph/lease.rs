use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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

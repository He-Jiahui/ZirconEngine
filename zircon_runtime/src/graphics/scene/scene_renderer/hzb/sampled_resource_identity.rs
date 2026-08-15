use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct HzbSampledResourceIdentity(u64);

static NEXT_HZB_SAMPLED_RESOURCE_ID: AtomicU64 = AtomicU64::new(1);

impl HzbSampledResourceIdentity {
    pub(crate) fn new() -> Self {
        Self(NEXT_HZB_SAMPLED_RESOURCE_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub(super) const fn get(self) -> u64 {
        self.0
    }
}

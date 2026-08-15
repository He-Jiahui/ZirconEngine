use std::sync::atomic::{AtomicU64, Ordering};

/// Stable identity for a concrete sampled texture backing while it remains alive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SampledTextureIdentity(u64);

static NEXT_SAMPLED_TEXTURE_IDENTITY: AtomicU64 = AtomicU64::new(1);

impl SampledTextureIdentity {
    pub(crate) fn new() -> Self {
        Self(NEXT_SAMPLED_TEXTURE_IDENTITY.fetch_add(1, Ordering::Relaxed))
    }

    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

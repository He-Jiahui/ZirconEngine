use std::num::NonZeroU64;

/// Monotonically distinguishes surface bindings for the same live window and
/// viewport. A new lease is required after every successful replacement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceLeaseGeneration(NonZeroU64);

impl SurfaceLeaseGeneration {
    pub(crate) const fn new(raw: u64) -> Option<Self> {
        match NonZeroU64::new(raw) {
            Some(raw) => Some(Self(raw)),
            None => None,
        }
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

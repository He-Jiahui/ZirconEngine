use std::num::NonZeroU64;

/// Monotonically identifies the observed window state consumed by a terminal
/// command receipt. It intentionally differs from the window handle
/// generation, which only changes when a registry slot is reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowObservedGeneration(NonZeroU64);

impl WindowObservedGeneration {
    pub(crate) const fn initial() -> Self {
        Self(NonZeroU64::MIN)
    }

    pub(crate) const fn new(raw: NonZeroU64) -> Self {
        Self(raw)
    }

    pub(crate) const fn next(self) -> Option<Self> {
        match self.0.get().checked_add(1) {
            Some(raw) => match NonZeroU64::new(raw) {
                Some(raw) => Some(Self(raw)),
                None => None,
            },
            None => None,
        }
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

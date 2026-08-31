use std::num::NonZeroU64;

/// Process-local identity allocated by the platform host for one window
/// command submission. It is separate from the target window generation so
/// retries and late terminal receipts cannot be confused with one another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowCommandId(NonZeroU64);

impl WindowCommandId {
    pub(crate) const fn new(raw: u64) -> Option<Self> {
        match NonZeroU64::new(raw) {
            Some(raw) => Some(Self(raw)),
            None => None,
        }
    }

    pub const fn raw(self) -> u64 {
        self.0.get()
    }
}

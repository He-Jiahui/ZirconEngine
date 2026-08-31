use std::num::NonZeroU64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlatformHostOperationId(NonZeroU64);

impl PlatformHostOperationId {
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

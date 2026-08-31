use std::num::NonZeroU64;

/// Identifies one installed platform-host owner within a process lifetime.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlatformHostInstanceId(NonZeroU64);

impl PlatformHostInstanceId {
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

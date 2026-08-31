use std::num::NonZeroU64;

/// Identifies one platform-host window registry within the running process.
///
/// It prevents a handle obtained from an old or different host instance from
/// addressing a live window in the current host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowRegistryId(NonZeroU64);

impl WindowRegistryId {
    pub const fn new(raw: u64) -> Option<Self> {
        match NonZeroU64::new(raw) {
            Some(raw) => Some(Self(raw)),
            None => None,
        }
    }

    pub const fn raw(self) -> u64 {
        self.0.get()
    }
}

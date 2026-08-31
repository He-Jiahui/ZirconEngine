use std::num::NonZeroU64;

/// Opaque identifier assigned by the active platform backend to one live
/// native window. It is intentionally not serializable or reusable across
/// platform-host instances.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NativeWindowId(NonZeroU64);

impl NativeWindowId {
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

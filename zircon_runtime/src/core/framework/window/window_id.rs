use std::num::NonZeroU32;

use super::WindowRegistryId;

/// A non-persistent, generation-qualified address of a live platform window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId {
    registry: WindowRegistryId,
    slot: u32,
    generation: NonZeroU32,
}

impl WindowId {
    pub(crate) const fn new(registry: WindowRegistryId, slot: u32, generation: NonZeroU32) -> Self {
        Self {
            registry,
            slot,
            generation,
        }
    }

    pub const fn registry(self) -> WindowRegistryId {
        self.registry
    }

    pub const fn slot(self) -> u32 {
        self.slot
    }

    pub const fn generation(self) -> u32 {
        self.generation.get()
    }
}

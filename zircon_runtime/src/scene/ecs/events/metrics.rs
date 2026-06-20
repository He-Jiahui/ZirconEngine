use std::mem::size_of;

pub const EVENT_INLINE_PAYLOAD_MAX_BYTES: usize = 128;
pub const EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventPayloadStorage {
    Inline,
    IndirectRecommended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventPayloadProfile {
    size_bytes: usize,
    storage: EventPayloadStorage,
}

impl EventPayloadProfile {
    pub const fn for_size(size_bytes: usize) -> Self {
        let storage = if size_bytes > EVENT_INLINE_PAYLOAD_MAX_BYTES {
            EventPayloadStorage::IndirectRecommended
        } else {
            EventPayloadStorage::Inline
        };
        Self {
            size_bytes,
            storage,
        }
    }

    pub fn of<T>() -> Self {
        Self::for_size(size_of::<T>())
    }

    pub const fn size_bytes(self) -> usize {
        self.size_bytes
    }

    pub const fn storage(self) -> EventPayloadStorage {
        self.storage
    }

    pub const fn requires_indirection(self) -> bool {
        match self.storage {
            EventPayloadStorage::Inline => false,
            EventPayloadStorage::IndirectRecommended => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EventCapacityMetrics {
    pub current_len: usize,
    pub next_len: usize,
    pub current_capacity: usize,
    pub next_capacity: usize,
    pub high_water_len: usize,
    pub low_water_frames: u32,
    pub shrink_count: u64,
}

impl EventCapacityMetrics {
    pub const fn retained_capacity(self) -> usize {
        if self.current_capacity > self.next_capacity {
            self.current_capacity
        } else {
            self.next_capacity
        }
    }

    pub const fn queued_len(self) -> usize {
        self.current_len + self.next_len
    }
}

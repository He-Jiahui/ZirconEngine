use super::GamepadInputRejection;

/// Remaining admission capacity for one atomic physical-input batch.
/// Owned bytes include the input Vec's allocated slots and every owned String's capacity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputBatchLimits {
    pub max_events: usize,
    pub max_owned_bytes: usize,
}

impl InputBatchLimits {
    pub const fn new(max_events: usize, max_owned_bytes: usize) -> Self {
        Self {
            max_events,
            max_owned_bytes,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputBatchSize {
    pub events: usize,
    pub owned_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputBatchRejection {
    Empty,
    EventCapacity {
        requested: usize,
        remaining: usize,
    },
    OwnedByteCapacity {
        requested: usize,
        remaining: usize,
    },
    SizeOverflow,
    NonFinite {
        event_index: usize,
    },
    InvalidPayload {
        event_index: usize,
    },
    Gamepad {
        event_index: usize,
        reason: GamepadInputRejection,
    },
}

impl InputBatchRejection {
    pub const fn is_capacity(&self) -> bool {
        matches!(
            self,
            Self::EventCapacity { .. } | Self::OwnedByteCapacity { .. } | Self::SizeOverflow
        )
    }
}

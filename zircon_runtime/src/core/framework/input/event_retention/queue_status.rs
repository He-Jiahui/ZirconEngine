use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputEventQueueStatus {
    pub retained_events: u32,
    pub coalesced_events: u64,
}

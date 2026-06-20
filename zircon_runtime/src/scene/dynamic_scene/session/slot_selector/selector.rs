use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuntimeSessionSlotSelector {
    SlotId { slot_id: String },
    LatestUpdated,
    OldestUpdated,
    LatestUpdatedWithTag { tag: String },
    OldestUpdatedWithTag { tag: String },
}

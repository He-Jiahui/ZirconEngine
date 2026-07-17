use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputEventRecordingStatus {
    pub enabled: bool,
    pub capacity: u32,
    pub retained_records: u32,
    pub discarded_records: u64,
}

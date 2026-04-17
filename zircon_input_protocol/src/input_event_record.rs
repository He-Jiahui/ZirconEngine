use serde::{Deserialize, Serialize};

use crate::InputEvent;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputEventRecord {
    pub sequence: u64,
    pub timestamp_millis: u64,
    pub event: InputEvent,
}

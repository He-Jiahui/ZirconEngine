use serde::{Deserialize, Serialize};

use crate::ui::dispatch::{UiInputSequence, UiInputTimestamp, UiWindowId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiWindowEventMetadata {
    pub window_id: UiWindowId,
    pub timestamp: UiInputTimestamp,
    pub sequence: UiInputSequence,
    pub synthetic: bool,
}

impl UiWindowEventMetadata {
    pub const fn for_window(
        window_id: UiWindowId,
        timestamp: UiInputTimestamp,
        sequence: UiInputSequence,
    ) -> Self {
        Self {
            window_id,
            timestamp,
            sequence,
            synthetic: false,
        }
    }

    pub const fn synthetic(mut self, synthetic: bool) -> Self {
        self.synthetic = synthetic;
        self
    }
}

impl Default for UiWindowEventMetadata {
    fn default() -> Self {
        Self {
            window_id: UiWindowId::default(),
            timestamp: UiInputTimestamp::default(),
            sequence: UiInputSequence::default(),
            synthetic: false,
        }
    }
}

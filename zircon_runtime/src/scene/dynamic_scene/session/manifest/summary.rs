use serde::{Deserialize, Serialize};

use super::super::RuntimeSessionMetadata;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSessionSlotSummary {
    pub slot_id: String,
    #[serde(default)]
    pub metadata: RuntimeSessionMetadata,
    pub scene_format_version: u32,
    pub entity_count: usize,
    pub resource_count: usize,
}

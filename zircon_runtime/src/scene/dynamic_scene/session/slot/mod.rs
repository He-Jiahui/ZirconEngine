mod capture;
mod diff;
mod restore;
mod scene_document;
mod summary;

use serde::{Deserialize, Serialize};

use super::super::DynamicScene;
use super::RuntimeSessionMetadata;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeSessionSlot {
    pub slot_id: String,
    #[serde(default)]
    pub metadata: RuntimeSessionMetadata,
    #[serde(with = "scene_document")]
    pub scene: DynamicScene,
}

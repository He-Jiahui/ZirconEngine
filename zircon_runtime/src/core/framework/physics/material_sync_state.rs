use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;

use super::PhysicsMaterialMetadata;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsMaterialSyncState {
    pub entity: EntityId,
    pub locator: Option<String>,
    pub material: PhysicsMaterialMetadata,
}

use crate::scene::{LevelSystem, World};

use super::super::super::DynamicScene;
use super::super::{
    slot_id::normalize_slot_id, RuntimeSessionArchiveError, RuntimeSessionMetadata,
};
use super::RuntimeSessionSlot;

impl RuntimeSessionSlot {
    pub fn from_world(
        slot_id: impl Into<String>,
        world: &World,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        Self::from_world_with_metadata(slot_id, world, RuntimeSessionMetadata::default())
    }

    pub fn from_world_with_metadata(
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        let slot_id = normalize_slot_id(slot_id.into())?;
        Ok(Self {
            slot_id,
            metadata: metadata.normalized(),
            scene: DynamicScene::from_world(world)?,
        })
    }

    pub fn from_level(
        slot_id: impl Into<String>,
        level: &LevelSystem,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        let metadata = RuntimeSessionMetadata::from_level_metadata(level.metadata());
        level.with_world(|world| Self::from_world_with_metadata(slot_id, world, metadata))
    }
}

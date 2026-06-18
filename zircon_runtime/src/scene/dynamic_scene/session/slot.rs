use serde::{Deserialize, Serialize};

use crate::scene::{LevelSystem, World};

use super::super::{DynamicScene, EntityRemap};
use super::slot_id::normalize_slot_id;
use super::{
    RuntimeSessionArchiveError, RuntimeSessionLevelRestoreReport, RuntimeSessionMetadata,
    RuntimeSessionSlotDiffReport, RuntimeSessionSlotSummary,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeSessionSlot {
    pub slot_id: String,
    #[serde(default)]
    pub metadata: RuntimeSessionMetadata,
    pub scene: DynamicScene,
}

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
        Self::from_world_with_metadata(slot_id, &level.snapshot(), metadata)
    }

    pub fn apply_to_world(
        &self,
        world: &mut World,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        Ok(self.scene.spawn_into(world)?)
    }

    pub fn restore_to_empty_world(&self) -> Result<World, RuntimeSessionArchiveError> {
        let mut world = World::empty();
        self.apply_to_world(&mut world)?;
        Ok(world)
    }

    pub fn restore_into_level(
        &self,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionLevelRestoreReport, RuntimeSessionArchiveError> {
        let world = self.restore_to_empty_world()?;
        let entity_count = world.node_records().len();
        let metadata = self.metadata.to_level_metadata();
        level.replace_world_and_reset_runtime_state(world);
        level.set_metadata(metadata.clone());
        Ok(RuntimeSessionLevelRestoreReport {
            slot_id: self.slot_id.clone(),
            metadata,
            entity_count,
        })
    }

    pub fn apply_to_level(
        &self,
        level: &LevelSystem,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        level.with_world_mut(|world| self.apply_to_world(world))
    }

    pub fn diff_world(
        &self,
        world: &World,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        let target_scene = DynamicScene::from_world(world)?;
        Ok(RuntimeSessionSlotDiffReport {
            slot_id: self.slot_id.clone(),
            matches: self.scene == target_scene,
            slot_entity_count: self.scene.entities.len(),
            target_entity_count: target_scene.entities.len(),
            slot_resource_count: self.scene.resources.len(),
            target_resource_count: target_scene.resources.len(),
        })
    }

    pub fn diff_level(
        &self,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        self.diff_world(&level.snapshot())
    }

    pub fn summary(&self) -> RuntimeSessionSlotSummary {
        RuntimeSessionSlotSummary {
            slot_id: self.slot_id.clone(),
            metadata: self.metadata.clone().normalized(),
            scene_format_version: self.scene.format_version,
            entity_count: self.scene.entities.len(),
            resource_count: self.scene.resources.len(),
        }
    }
}

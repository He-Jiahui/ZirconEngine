use crate::scene::{LevelSystem, World};

use super::super::super::DynamicScene;
use super::super::{RuntimeSessionArchiveError, RuntimeSessionSlotDiffReport};
use super::RuntimeSessionSlot;

impl RuntimeSessionSlot {
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
}

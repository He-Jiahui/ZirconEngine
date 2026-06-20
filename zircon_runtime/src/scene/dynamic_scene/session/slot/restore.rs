use crate::scene::{LevelSystem, World};

use super::super::super::EntityRemap;
use super::super::{RuntimeSessionArchiveError, RuntimeSessionLevelRestoreReport};
use super::RuntimeSessionSlot;

impl RuntimeSessionSlot {
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
}

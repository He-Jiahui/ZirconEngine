use crate::scene::{LevelSystem, World};

use super::super::super::super::super::EntityRemap;
use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, restore as session_restore,
};

impl RuntimeSessionArchive {
    pub fn apply_slot(
        &self,
        slot_id: &str,
        world: &mut World,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        session_restore::apply_slot(self, slot_id, world)
    }

    pub fn apply_slot_to_level(
        &self,
        slot_id: &str,
        level: &LevelSystem,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        session_restore::apply_slot_to_level(self, slot_id, level)
    }
}

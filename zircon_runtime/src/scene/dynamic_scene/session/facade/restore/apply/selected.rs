use crate::scene::{LevelSystem, World};

use super::super::super::super::super::EntityRemap;
use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector,
    restore as session_restore,
};

impl RuntimeSessionArchive {
    pub fn apply_selected_slot(
        &self,
        selector: RuntimeSessionSlotSelector,
        world: &mut World,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        session_restore::apply_selected_slot(self, selector, world)
    }

    pub fn apply_selected_slot_to_level(
        &self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        session_restore::apply_selected_slot_to_level(self, selector, level)
    }
}

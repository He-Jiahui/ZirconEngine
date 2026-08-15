use crate::scene::{LevelSystem, World};

use super::super::super::super::{
    restore as session_restore, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionLevelRestoreReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn restore_selected_slot_to_empty_world(
        &self,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<World, RuntimeSessionArchiveError> {
        session_restore::restore_selected_slot_to_empty_world(self, selector)
    }

    pub fn restore_selected_slot_into_level(
        &self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionLevelRestoreReport, RuntimeSessionArchiveError> {
        session_restore::restore_selected_slot_into_level(self, selector, level)
    }
}

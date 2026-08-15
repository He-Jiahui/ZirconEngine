use crate::scene::{LevelSystem, World};

use super::super::super::super::{
    restore as session_restore, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotDiffReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn diff_selected_slot_with_world(
        &self,
        selector: RuntimeSessionSlotSelector,
        world: &World,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        session_restore::diff_selected_slot_with_world(self, selector, world)
    }

    pub fn diff_selected_slot_with_level(
        &self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        session_restore::diff_selected_slot_with_level(self, selector, level)
    }
}

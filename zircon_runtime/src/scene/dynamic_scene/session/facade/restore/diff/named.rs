use crate::scene::{LevelSystem, World};

use super::super::super::super::{
    restore as session_restore, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotDiffReport,
};

impl RuntimeSessionArchive {
    pub fn diff_slot_with_world(
        &self,
        slot_id: &str,
        world: &World,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        session_restore::diff_slot_with_world(self, slot_id, world)
    }

    pub fn diff_slot_with_level(
        &self,
        slot_id: &str,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        session_restore::diff_slot_with_level(self, slot_id, level)
    }
}

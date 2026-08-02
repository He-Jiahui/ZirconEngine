use crate::scene::{LevelSystem, World};

use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionLevelRestoreReport,
    restore as session_restore,
};

impl RuntimeSessionArchive {
    pub fn restore_slot_to_empty_world(
        &self,
        slot_id: &str,
    ) -> Result<World, RuntimeSessionArchiveError> {
        session_restore::restore_slot_to_empty_world(self, slot_id)
    }

    pub fn restore_slot_into_level(
        &self,
        slot_id: &str,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionLevelRestoreReport, RuntimeSessionArchiveError> {
        session_restore::restore_slot_into_level(self, slot_id, level)
    }
}

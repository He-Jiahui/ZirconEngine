use crate::scene::LevelSystem;

use super::super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn capture_level_slot(
        &mut self,
        slot_id: impl Into<String>,
        level: &LevelSystem,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_capture::capture_level_slot(self, slot_id, level)
    }
}

use crate::scene::LevelSystem;

use super::super::super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn capture_level_slot_with_retention(
        &mut self,
        slot_id: impl Into<String>,
        level: &LevelSystem,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        capture_retention::capture_level_slot_with_retention(self, slot_id, level, policy)
    }
}

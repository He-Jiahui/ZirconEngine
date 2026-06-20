use crate::scene::LevelSystem;

use super::super::super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn preview_capture_level_slot_with_tag_retention(
        &self,
        tag: &str,
        slot_id: impl Into<String>,
        level: &LevelSystem,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        capture_retention::preview_level_slot_with_tag_retention(self, tag, slot_id, level, policy)
    }
}

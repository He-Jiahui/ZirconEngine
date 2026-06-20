use crate::scene::LevelSystem;

use super::super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn preview_capture_level_slot(
        &self,
        slot_id: impl Into<String>,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        slot_capture::preview_level_slot(self, slot_id, level).map(|preview| preview.report)
    }
}

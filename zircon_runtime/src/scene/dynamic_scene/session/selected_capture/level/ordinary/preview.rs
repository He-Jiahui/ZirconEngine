use crate::scene::LevelSystem;

use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotCapturePreviewReport,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_capture_level_selected_slot(
        &self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_capture_level_slot(report.selected_slot_id, level)
    }

    pub fn preview_capture_level_selected_slot_preserving_metadata(
        &self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        let world = level.snapshot();
        self.preview_capture_world_slot(report.selected_slot_id, &world, report.summary.metadata)
    }
}

use crate::scene::World;

use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotCapturePreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_capture_world_selected_slot(
        &self,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_capture_world_slot(report.selected_slot_id, world, metadata)
    }

    pub fn preview_capture_world_selected_slot_preserving_metadata(
        &self,
        selector: RuntimeSessionSlotSelector,
        world: &World,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_capture_world_slot(report.selected_slot_id, world, report.summary.metadata)
    }
}

use crate::scene::World;

use super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_capture_world_selected_slot_with_retention(
        &self,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        metadata: RuntimeSessionMetadata,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_capture_world_slot_with_retention(
            report.selected_slot_id,
            world,
            metadata,
            policy,
        )
    }

    pub fn preview_capture_world_selected_slot_preserving_metadata_with_retention(
        &self,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_capture_world_slot_with_retention(
            report.selected_slot_id,
            world,
            report.summary.metadata,
            policy,
        )
    }
}

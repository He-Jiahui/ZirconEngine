use crate::scene::World;

use super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn capture_world_selected_slot_with_retention(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        metadata: RuntimeSessionMetadata,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.capture_world_slot_with_retention(report.selected_slot_id, world, metadata, policy)
    }

    pub fn capture_world_selected_slot_preserving_metadata_with_retention(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.capture_world_slot_with_retention(
            report.selected_slot_id,
            world,
            report.summary.metadata,
            policy,
        )
    }
}

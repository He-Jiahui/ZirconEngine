use crate::scene::LevelSystem;

use super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn capture_level_selected_slot_with_tag_retention(
        &mut self,
        tag: &str,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.capture_level_slot_with_tag_retention(tag, report.selected_slot_id, level, policy)
    }

    pub fn capture_level_selected_slot_preserving_metadata_with_tag_retention(
        &mut self,
        tag: &str,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        let world = level.snapshot();
        self.capture_world_slot_with_tag_retention(
            tag,
            report.selected_slot_id,
            &world,
            report.summary.metadata,
            policy,
        )
    }
}

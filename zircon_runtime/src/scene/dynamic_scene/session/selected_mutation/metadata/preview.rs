use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotMutationPreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_update_selected_slot_metadata(
        &self,
        selector: RuntimeSessionSlotSelector,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_update_slot_metadata(&report.selected_slot_id, metadata)
    }
}

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotMutationPreviewReport,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_touch_selected_slot(
        &self,
        selector: RuntimeSessionSlotSelector,
        updated_at_unix_millis: u64,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_touch_slot(&report.selected_slot_id, updated_at_unix_millis)
    }
}

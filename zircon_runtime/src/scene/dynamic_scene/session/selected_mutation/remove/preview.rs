use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotMutationPreviewReport,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_remove_selected_slot(
        &self,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_remove_slot(&report.selected_slot_id)
    }
}

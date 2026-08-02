use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
    RuntimeSessionSlotSelector, slot_copy,
};

impl RuntimeSessionArchive {
    pub fn preview_copy_selected_slot(
        &self,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        slot_copy::preview_copy_selected_slot(self, selector, new_slot_id)
    }
}

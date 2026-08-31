use super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotExportPreviewReport,
    RuntimeSessionSlotSelector, slot_export,
};

impl RuntimeSessionArchive {
    pub fn preview_selected_single_slot_archive(
        &self,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
        slot_export::preview_selected_single_slot_archive(self, selector)
    }
}

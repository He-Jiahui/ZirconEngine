use super::super::super::super::super::{
    slot_export, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotExportPreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_selected_single_slot_archive(
        &self,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
        slot_export::preview_selected_single_slot_archive(self, selector)
    }
}

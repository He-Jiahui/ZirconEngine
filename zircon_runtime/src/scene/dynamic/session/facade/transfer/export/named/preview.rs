use super::super::super::super::super::{
    slot_export, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotExportPreviewReport,
};

impl RuntimeSessionArchive {
    pub fn preview_single_slot_archive(
        &self,
        slot_id: &str,
    ) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
        slot_export::preview_single_slot_archive(self, slot_id)
    }
}

use super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotExportPreviewReport,
    slot_export,
};

impl RuntimeSessionArchive {
    pub fn preview_single_slot_archive(
        &self,
        slot_id: &str,
    ) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
        slot_export::preview_single_slot_archive(self, slot_id)
    }
}

use std::path::Path;

use super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotExportPreviewReport,
    RuntimeSessionSlotSelector, path_export,
};

impl RuntimeSessionArchive {
    pub fn preview_save_selected_single_slot_archive_to_path(
        &self,
        selector: RuntimeSessionSlotSelector,
        target_path: impl AsRef<Path>,
    ) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
        path_export::preview_save_selected_single_slot_archive_to_path(self, selector, target_path)
    }
}

use std::path::Path;

use super::super::super::super::super::{
    path_export, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotExportPreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_save_selected_single_slot_archive_from_path(
        source_path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        target_path: impl AsRef<Path>,
    ) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
        path_export::preview_save_selected_single_slot_archive_from_path(
            source_path,
            selector,
            target_path,
        )
    }
}

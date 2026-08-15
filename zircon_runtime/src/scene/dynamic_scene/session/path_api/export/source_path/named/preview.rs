use std::path::Path;

use super::super::super::super::super::{
    path_export, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotExportPreviewReport,
};

impl RuntimeSessionArchive {
    pub fn preview_save_single_slot_archive_from_path(
        source_path: impl AsRef<Path>,
        slot_id: &str,
        target_path: impl AsRef<Path>,
    ) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
        path_export::preview_save_single_slot_archive_from_path(source_path, slot_id, target_path)
    }
}

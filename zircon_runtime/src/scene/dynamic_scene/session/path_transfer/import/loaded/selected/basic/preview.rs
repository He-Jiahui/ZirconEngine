use std::path::Path;

use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
    RuntimeSessionSlotSelector, io,
};

pub(in crate::scene::dynamic_scene::session) fn preview_import_selected_slot_from_archive_at_path(
    path: impl AsRef<Path>,
    incoming: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.preview_import_selected_slot_from_archive(
        incoming,
        selector,
        new_slot_id,
    )
}

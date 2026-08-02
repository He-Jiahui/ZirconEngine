use std::path::Path;

use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport, io,
};

pub(in crate::scene::dynamic_scene::session) fn preview_import_slot_from_archive_at_path(
    path: impl AsRef<Path>,
    incoming: &RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.preview_import_slot_from_archive(
        incoming,
        source_slot_id,
        new_slot_id,
    )
}

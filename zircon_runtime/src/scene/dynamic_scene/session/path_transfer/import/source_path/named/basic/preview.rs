use std::path::Path;

use super::super::super::super::super::super::{
    io, target_path as archive_target_path, RuntimeSessionArchiveError,
    RuntimeSessionSlotImportPreviewReport,
};
use super::super::super::super::loaded::preview_import_slot_from_archive_at_path;

pub(in crate::scene::dynamic_scene::session) fn preview_import_slot_from_archive_path_at_path(
    path: impl AsRef<Path>,
    source_path: impl AsRef<Path>,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    let path = path.as_ref();
    let source_path = source_path.as_ref();
    archive_target_path::reject_same_archive_paths(
        source_path,
        path,
        "runtime session single-slot archive import",
    )?;
    let incoming = io::load_from_path(source_path)?;
    preview_import_slot_from_archive_at_path(path, &incoming, source_slot_id, new_slot_id)
}

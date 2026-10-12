use std::path::Path;

use super::super::super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotSelector,
};

pub(in crate::scene::dynamic_scene::session) fn preview_import_selected_slot_from_archive_with_metadata_at_path(
    path: impl AsRef<Path>,
    incoming: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.preview_import_selected_slot_from_archive_with_metadata(
        incoming,
        selector,
        new_slot_id,
        metadata,
    )
}

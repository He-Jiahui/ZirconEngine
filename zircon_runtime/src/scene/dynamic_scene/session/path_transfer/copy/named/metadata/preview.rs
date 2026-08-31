use std::path::Path;

use super::super::super::super::super::{
    RuntimeSessionArchiveError, RuntimeSessionMetadata, RuntimeSessionSlotImportPreviewReport, io,
};

pub(in crate::scene::dynamic_scene::session) fn preview_copy_slot_with_metadata_from_path(
    path: impl AsRef<Path>,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.preview_copy_slot_with_metadata(source_slot_id, new_slot_id, metadata)
}

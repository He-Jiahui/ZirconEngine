use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
};
use super::super::metadata::preview_copy_slot_with_metadata;

pub(in crate::scene::dynamic_scene::session) fn preview_copy_slot(
    archive: &RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    let metadata = archive.require_slot(source_slot_id)?.metadata.clone();
    preview_copy_slot_with_metadata(archive, source_slot_id, new_slot_id, metadata)
}

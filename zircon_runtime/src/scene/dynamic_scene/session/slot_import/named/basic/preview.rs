use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
};
use super::super::metadata::preview_import_slot_from_archive_with_metadata;

pub(in crate::scene::dynamic_scene::session) fn preview_import_slot_from_archive(
    target: &RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    let metadata = incoming.require_slot(source_slot_id)?.metadata.clone();
    preview_import_slot_from_archive_with_metadata(
        target,
        incoming,
        source_slot_id,
        new_slot_id,
        metadata,
    )
}

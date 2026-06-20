use super::super::super::super::slot_id::normalize_slot_id;
use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport,
};

pub(in crate::scene::dynamic_scene::session) fn preview_copy_slot_with_metadata(
    archive: &RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;

    let destination_slot_id = normalize_slot_id(new_slot_id.into())?;
    if archive.contains_slot(&destination_slot_id) {
        return Err(RuntimeSessionArchiveError::DuplicateSlotId {
            slot_id: destination_slot_id,
        });
    }

    let slot =
        archive
            .slot(source_slot_id)
            .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
                slot_id: source_slot_id.to_string(),
            })?;

    Ok(RuntimeSessionSlotImportPreviewReport {
        source_slot_id: slot.slot_id.clone(),
        destination_slot_id,
        metadata: metadata.normalized(),
        entity_count: slot.scene.entities.len(),
        resource_count: slot.scene.resources.len(),
    })
}

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotMutationPreviewReport,
};

pub(in crate::scene::dynamic_scene::session) fn preview_update_slot_metadata(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;

    let slot = archive
        .slot(slot_id)
        .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
            slot_id: slot_id.to_string(),
        })?;

    Ok(RuntimeSessionSlotMutationPreviewReport {
        source_slot_id: slot.slot_id.clone(),
        destination_slot_id: None,
        metadata: metadata.normalized(),
        entity_count: slot.scene.entities.len(),
        resource_count: slot.scene.resources.len(),
    })
}

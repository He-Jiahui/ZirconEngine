use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotMutationPreviewReport,
};

pub(in crate::scene::dynamic_scene::session) fn preview_touch_slot(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
    updated_at_unix_millis: u64,
) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;

    let slot = archive
        .slot(slot_id)
        .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
            slot_id: slot_id.to_string(),
        })?;
    let mut metadata = slot.metadata.clone().normalized();
    metadata.updated_at_unix_millis = Some(updated_at_unix_millis);

    Ok(RuntimeSessionSlotMutationPreviewReport {
        source_slot_id: slot.slot_id.clone(),
        destination_slot_id: None,
        metadata,
        entity_count: slot.scene.entities.len(),
        resource_count: slot.scene.resources.len(),
    })
}

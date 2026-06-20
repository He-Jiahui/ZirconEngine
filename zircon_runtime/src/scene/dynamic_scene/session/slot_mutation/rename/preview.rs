use super::super::super::slot_id::normalize_slot_id;
use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotMutationPreviewReport,
};
use super::super::report::slot_mutation_report;

pub(in crate::scene::dynamic_scene::session) fn preview_rename_slot(
    archive: &RuntimeSessionArchive,
    old_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;

    let destination_slot_id = normalize_slot_id(new_slot_id.into())?;
    let slot =
        archive
            .slot(old_slot_id)
            .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
                slot_id: old_slot_id.to_string(),
            })?;

    if slot.slot_id.as_str() != destination_slot_id.as_str()
        && archive.contains_slot(&destination_slot_id)
    {
        return Err(RuntimeSessionArchiveError::DuplicateSlotId {
            slot_id: destination_slot_id,
        });
    }

    Ok(slot_mutation_report(slot, Some(destination_slot_id)))
}

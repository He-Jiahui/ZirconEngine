use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotMutationPreviewReport,
};
use super::super::report::slot_mutation_report;

pub(in crate::scene::dynamic_scene::session) fn preview_remove_slot(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;

    let slot = archive
        .slot(slot_id)
        .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
            slot_id: slot_id.to_string(),
        })?;

    Ok(slot_mutation_report(slot, None))
}

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
};
use super::preview::preview_update_slot_metadata;

pub(in crate::scene::dynamic_scene::session) fn update_slot_metadata(
    archive: &mut RuntimeSessionArchive,
    slot_id: &str,
    metadata: RuntimeSessionMetadata,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = preview_update_slot_metadata(archive, slot_id, metadata)?;
    let slot = archive.slot_mut(&report.source_slot_id).ok_or_else(|| {
        RuntimeSessionArchiveError::MissingSlot {
            slot_id: report.source_slot_id.clone(),
        }
    })?;
    slot.metadata = report.metadata;
    Ok(())
}

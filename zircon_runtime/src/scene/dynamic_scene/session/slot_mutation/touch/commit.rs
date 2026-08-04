use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::preview::preview_touch_slot;

pub(in crate::scene::dynamic_scene::session) fn touch_slot(
    archive: &mut RuntimeSessionArchive,
    slot_id: &str,
    updated_at_unix_millis: u64,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = preview_touch_slot(archive, slot_id, updated_at_unix_millis)?;
    let slot = archive.slot_mut(&report.source_slot_id).ok_or_else(|| {
        RuntimeSessionArchiveError::MissingSlot {
            slot_id: report.source_slot_id.clone(),
        }
    })?;
    slot.metadata = report.metadata;
    archive.rebuild_slot_indexes();
    Ok(())
}

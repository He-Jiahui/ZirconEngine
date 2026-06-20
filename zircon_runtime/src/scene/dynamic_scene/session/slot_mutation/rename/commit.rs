use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::preview::preview_rename_slot;

pub(in crate::scene::dynamic_scene::session) fn rename_slot(
    archive: &mut RuntimeSessionArchive,
    old_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = preview_rename_slot(archive, old_slot_id, new_slot_id)?;
    let new_slot_id = report
        .destination_slot_id
        .expect("rename slot preview always reports a destination slot id");
    let slot_index = archive
        .slots
        .iter()
        .position(|slot| slot.slot_id == report.source_slot_id)
        .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
            slot_id: report.source_slot_id.clone(),
        })?;

    archive.slots[slot_index].slot_id = new_slot_id;
    archive.sort_slots();
    Ok(())
}

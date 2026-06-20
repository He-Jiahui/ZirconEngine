use super::super::slot_id::validate_canonical_slot_id;
use super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot};

pub(in crate::scene::dynamic_scene::session) fn upsert_slot(
    archive: &mut RuntimeSessionArchive,
    mut slot: RuntimeSessionSlot,
) -> Result<(), RuntimeSessionArchiveError> {
    validate_canonical_slot_id(&slot.slot_id)?;
    slot.metadata.normalize();
    slot.scene.ensure_supported()?;
    if let Some(existing) = archive
        .slots
        .iter_mut()
        .find(|existing| existing.slot_id == slot.slot_id)
    {
        *existing = slot;
    } else {
        archive.slots.push(slot);
    }
    archive.sort_slots();
    Ok(())
}

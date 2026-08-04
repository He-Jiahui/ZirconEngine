use super::super::slot_id::validate_canonical_slot_id;
use super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot};

pub(in crate::scene::dynamic_scene::session) fn upsert_slot(
    archive: &mut RuntimeSessionArchive,
    mut slot: RuntimeSessionSlot,
) -> Result<(), RuntimeSessionArchiveError> {
    validate_canonical_slot_id(&slot.slot_id)?;
    slot.metadata.normalize();
    slot.scene.ensure_supported()?;
    archive.commit_slot_upsert(slot);
    Ok(())
}

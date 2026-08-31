use std::collections::BTreeSet;

use super::super::slot_id::validate_canonical_slot_id;
use super::super::{
    RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION, RuntimeSessionArchive, RuntimeSessionArchiveError,
};

pub(in crate::scene::dynamic_scene::session) fn ensure_supported(
    archive: &RuntimeSessionArchive,
) -> Result<(), RuntimeSessionArchiveError> {
    if archive.format_version != RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION {
        return Err(RuntimeSessionArchiveError::UnsupportedFormatVersion {
            expected: RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
            actual: archive.format_version,
        });
    }

    let mut seen = BTreeSet::new();
    for slot in archive.iter_dense_slot_rows() {
        validate_canonical_slot_id(&slot.slot_id)?;
        slot.scene.ensure_supported()?;
        if !seen.insert(slot.slot_id.as_str()) {
            return Err(RuntimeSessionArchiveError::DuplicateSlotId {
                slot_id: slot.slot_id.clone(),
            });
        }
    }
    Ok(())
}

use std::collections::BTreeSet;

use super::super::slot_id::validate_canonical_slot_id;
use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
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
    // 必须检查原始密集行；按 ID 建立的索引可能覆盖重复键，不能用索引视图证明槽位唯一。
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

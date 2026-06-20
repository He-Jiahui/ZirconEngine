use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot,
    RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
};

pub(in crate::scene::dynamic_scene::session) fn empty() -> RuntimeSessionArchive {
    RuntimeSessionArchive {
        format_version: RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
        slots: Vec::new(),
    }
}

pub(in crate::scene::dynamic_scene::session) fn from_slots(
    slots: Vec<RuntimeSessionSlot>,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    let mut archive = RuntimeSessionArchive {
        format_version: RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
        slots,
    };
    archive.normalize_slot_metadata();
    archive.sort_slots();
    archive.ensure_supported()?;
    Ok(archive)
}

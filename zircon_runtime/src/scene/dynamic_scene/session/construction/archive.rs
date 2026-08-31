use super::super::{
    RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlot,
};

pub(in crate::scene::dynamic_scene::session) fn empty() -> RuntimeSessionArchive {
    let archive =
        RuntimeSessionArchive::from_payload(RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION, Vec::new());
    archive.record_normalized();
    archive
}

pub(in crate::scene::dynamic_scene::session) fn from_slots(
    slots: Vec<RuntimeSessionSlot>,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    let mut archive =
        RuntimeSessionArchive::from_payload(RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION, slots);
    archive.normalize_slot_metadata();
    archive.record_normalized();
    archive.ensure_supported()?;
    archive.record_validated();
    Ok(archive)
}

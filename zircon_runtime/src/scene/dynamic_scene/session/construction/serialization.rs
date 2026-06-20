use super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};

pub(in crate::scene::dynamic_scene::session) fn from_versioned_json(
    json: &str,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    let mut archive: RuntimeSessionArchive = serde_json::from_str(json)?;
    archive.normalize_slot_metadata();
    archive.ensure_supported()?;
    archive.sort_slots();
    Ok(archive)
}

pub(in crate::scene::dynamic_scene::session) fn to_versioned_json_pretty(
    archive: &RuntimeSessionArchive,
) -> Result<String, RuntimeSessionArchiveError> {
    let mut archive = archive.clone();
    archive.normalize_slot_metadata();
    archive.sort_slots();
    archive.ensure_supported()?;
    Ok(serde_json::to_string_pretty(&archive)?)
}

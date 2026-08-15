use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};

pub(in crate::scene::dynamic_scene::session) fn latest_updated_slot_id_with_tag(
    archive: &RuntimeSessionArchive,
    tag: &str,
) -> Result<Option<String>, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    let tag = tag.trim();
    if tag.is_empty() {
        return Ok(None);
    }
    Ok(archive
        .indexed_latest_tag_slot(tag)
        .map(|slot| slot.slot_id.clone()))
}

pub(in crate::scene::dynamic_scene::session) fn oldest_updated_slot_id_with_tag(
    archive: &RuntimeSessionArchive,
    tag: &str,
) -> Result<Option<String>, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    let tag = tag.trim();
    if tag.is_empty() {
        return Ok(None);
    }
    Ok(archive
        .indexed_oldest_tag_slot(tag)
        .map(|slot| slot.slot_id.clone()))
}

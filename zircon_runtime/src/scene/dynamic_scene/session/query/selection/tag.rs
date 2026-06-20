use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::manifest::manifest;

pub(in crate::scene::dynamic_scene::session) fn latest_updated_slot_id_with_tag(
    archive: &RuntimeSessionArchive,
    tag: &str,
) -> Result<Option<String>, RuntimeSessionArchiveError> {
    Ok(manifest(archive)?
        .latest_updated_slot_with_tag(tag)
        .map(|slot| slot.slot_id.clone()))
}

pub(in crate::scene::dynamic_scene::session) fn oldest_updated_slot_id_with_tag(
    archive: &RuntimeSessionArchive,
    tag: &str,
) -> Result<Option<String>, RuntimeSessionArchiveError> {
    Ok(manifest(archive)?
        .oldest_updated_slot_with_tag(tag)
        .map(|slot| slot.slot_id.clone()))
}

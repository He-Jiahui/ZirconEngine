use std::path::Path;

use super::super::{
    RuntimeSessionArchiveError, RuntimeSessionArchiveManifest, RuntimeSessionSlotSummary,
    io as archive_io,
};

pub(in crate::scene::dynamic_scene::session) fn load_manifest_from_path(
    path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    archive_io::load_from_path(path)?.manifest()
}

pub(in crate::scene::dynamic_scene::session) fn slot_summary_from_path(
    path: impl AsRef<Path>,
    slot_id: &str,
) -> Result<Option<RuntimeSessionSlotSummary>, RuntimeSessionArchiveError> {
    Ok(load_manifest_from_path(path)?.slot(slot_id).cloned())
}

pub(in crate::scene::dynamic_scene::session) fn contains_slot_from_path(
    path: impl AsRef<Path>,
    slot_id: &str,
) -> Result<bool, RuntimeSessionArchiveError> {
    Ok(load_manifest_from_path(path)?.slot(slot_id).is_some())
}

pub(in crate::scene::dynamic_scene::session) fn slot_ids_from_path(
    path: impl AsRef<Path>,
) -> Result<Vec<String>, RuntimeSessionArchiveError> {
    Ok(load_manifest_from_path(path)?
        .slot_ids()
        .map(str::to_string)
        .collect())
}

pub(in crate::scene::dynamic_scene::session) fn slots_with_tag_from_path(
    path: impl AsRef<Path>,
    tag: &str,
) -> Result<Vec<RuntimeSessionSlotSummary>, RuntimeSessionArchiveError> {
    Ok(load_manifest_from_path(path)?
        .slots_with_tag(tag)
        .cloned()
        .collect())
}

pub(in crate::scene::dynamic_scene::session) fn slots_matching_display_name_from_path(
    path: impl AsRef<Path>,
    query: &str,
) -> Result<Vec<RuntimeSessionSlotSummary>, RuntimeSessionArchiveError> {
    Ok(load_manifest_from_path(path)?
        .slots_matching_display_name(query)
        .cloned()
        .collect())
}

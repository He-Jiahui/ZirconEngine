use std::path::Path;

use super::super::{
    RuntimeSessionArchiveError, RuntimeSessionSlotSelectionReport, RuntimeSessionSlotSelector,
    io as archive_io,
};

pub(in crate::scene::dynamic_scene::session) fn latest_updated_slot_id_from_path(
    path: impl AsRef<Path>,
) -> Result<Option<String>, RuntimeSessionArchiveError> {
    archive_io::load_from_path(path)?.latest_updated_slot_id()
}

pub(in crate::scene::dynamic_scene::session) fn oldest_updated_slot_id_from_path(
    path: impl AsRef<Path>,
) -> Result<Option<String>, RuntimeSessionArchiveError> {
    archive_io::load_from_path(path)?.oldest_updated_slot_id()
}

pub(in crate::scene::dynamic_scene::session) fn latest_updated_slot_id_with_tag_from_path(
    path: impl AsRef<Path>,
    tag: &str,
) -> Result<Option<String>, RuntimeSessionArchiveError> {
    archive_io::load_from_path(path)?.latest_updated_slot_id_with_tag(tag)
}

pub(in crate::scene::dynamic_scene::session) fn oldest_updated_slot_id_with_tag_from_path(
    path: impl AsRef<Path>,
    tag: &str,
) -> Result<Option<String>, RuntimeSessionArchiveError> {
    archive_io::load_from_path(path)?.oldest_updated_slot_id_with_tag(tag)
}

pub(in crate::scene::dynamic_scene::session) fn select_slot_from_path(
    path: impl AsRef<Path>,
    selector: RuntimeSessionSlotSelector,
) -> Result<RuntimeSessionSlotSelectionReport, RuntimeSessionArchiveError> {
    archive_io::load_from_path(path)?.select_slot(selector)
}

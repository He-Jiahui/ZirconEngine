use std::path::Path;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelectionReport,
    RuntimeSessionSlotSelector, path_query,
};

impl RuntimeSessionArchive {
    pub fn latest_updated_slot_id_from_path(
        path: impl AsRef<Path>,
    ) -> Result<Option<String>, RuntimeSessionArchiveError> {
        path_query::latest_updated_slot_id_from_path(path)
    }

    pub fn oldest_updated_slot_id_from_path(
        path: impl AsRef<Path>,
    ) -> Result<Option<String>, RuntimeSessionArchiveError> {
        path_query::oldest_updated_slot_id_from_path(path)
    }

    pub fn latest_updated_slot_id_with_tag_from_path(
        path: impl AsRef<Path>,
        tag: &str,
    ) -> Result<Option<String>, RuntimeSessionArchiveError> {
        path_query::latest_updated_slot_id_with_tag_from_path(path, tag)
    }

    pub fn oldest_updated_slot_id_with_tag_from_path(
        path: impl AsRef<Path>,
        tag: &str,
    ) -> Result<Option<String>, RuntimeSessionArchiveError> {
        path_query::oldest_updated_slot_id_with_tag_from_path(path, tag)
    }

    pub fn select_slot_from_path(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionSlotSelectionReport, RuntimeSessionArchiveError> {
        path_query::select_slot_from_path(path, selector)
    }
}

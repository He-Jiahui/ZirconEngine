use std::path::Path;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionSlotSummary, path_query,
};

impl RuntimeSessionArchive {
    pub fn load_manifest_from_path(
        path: impl AsRef<Path>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_query::load_manifest_from_path(path)
    }

    pub fn slot_summary_from_path(
        path: impl AsRef<Path>,
        slot_id: &str,
    ) -> Result<Option<RuntimeSessionSlotSummary>, RuntimeSessionArchiveError> {
        path_query::slot_summary_from_path(path, slot_id)
    }

    pub fn contains_slot_from_path(
        path: impl AsRef<Path>,
        slot_id: &str,
    ) -> Result<bool, RuntimeSessionArchiveError> {
        path_query::contains_slot_from_path(path, slot_id)
    }

    pub fn slot_ids_from_path(
        path: impl AsRef<Path>,
    ) -> Result<Vec<String>, RuntimeSessionArchiveError> {
        path_query::slot_ids_from_path(path)
    }

    pub fn slots_with_tag_from_path(
        path: impl AsRef<Path>,
        tag: &str,
    ) -> Result<Vec<RuntimeSessionSlotSummary>, RuntimeSessionArchiveError> {
        path_query::slots_with_tag_from_path(path, tag)
    }

    pub fn slots_matching_display_name_from_path(
        path: impl AsRef<Path>,
        query: &str,
    ) -> Result<Vec<RuntimeSessionSlotSummary>, RuntimeSessionArchiveError> {
        path_query::slots_matching_display_name_from_path(path, query)
    }
}

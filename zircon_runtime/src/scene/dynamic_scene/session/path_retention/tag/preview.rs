use std::path::Path;

use super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_prune_slots_with_tag_from_path(
        path: impl AsRef<Path>,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::load_from_path(path)?.preview_prune_slots_with_tag(tag, policy)
    }

    pub fn preview_prune_slots_with_tag_and_selected_protection_from_path(
        path: impl AsRef<Path>,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::load_from_path(path)?
            .preview_prune_slots_with_tag_and_selected_protection(tag, policy, selector)
    }
}

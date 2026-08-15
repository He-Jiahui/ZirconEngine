use std::path::Path;

use super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn prune_slots_with_tag_at_path_atomically(
        path: impl AsRef<Path>,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_with_report_atomically(path, |archive| {
            archive.prune_slots_with_tag(tag, policy)
        })
    }

    pub fn prune_slots_with_tag_and_selected_protection_at_path_atomically(
        path: impl AsRef<Path>,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_with_report_atomically(path, |archive| {
            archive.prune_slots_with_tag_and_selected_protection(tag, policy, selector)
        })
    }
}

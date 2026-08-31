use std::path::Path;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector, io,
};

impl RuntimeSessionArchive {
    pub fn prune_slots_at_path_atomically(
        path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_with_report_atomically(path, |archive| {
            archive.prune_slots(policy)
        })
    }

    pub fn prune_slots_with_selected_protection_at_path_atomically(
        path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveRetentionPolicy,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_with_report_atomically(path, |archive| {
            archive.prune_slots_with_selected_protection(policy, selector)
        })
    }
}

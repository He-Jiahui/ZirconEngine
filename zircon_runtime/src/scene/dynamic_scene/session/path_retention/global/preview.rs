use std::path::Path;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector, io,
};

impl RuntimeSessionArchive {
    pub fn preview_prune_slots_from_path(
        path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::load_from_path(path)?.preview_prune_slots(policy)
    }

    pub fn preview_prune_slots_with_selected_protection_from_path(
        path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveRetentionPolicy,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::load_from_path(path)?.preview_prune_slots_with_selected_protection(policy, selector)
    }
}

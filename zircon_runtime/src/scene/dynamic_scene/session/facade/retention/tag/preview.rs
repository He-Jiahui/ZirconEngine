use super::super::super::super::retention;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn preview_prune_slots_with_tag(
        &self,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        retention::preview_prune_slots_with_tag(self, tag, policy)
    }
}

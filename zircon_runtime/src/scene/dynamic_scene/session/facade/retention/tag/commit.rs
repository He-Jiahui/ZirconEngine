use super::super::super::super::retention;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn prune_slots_with_tag(
        &mut self,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        retention::prune_slots_with_tag(self, tag, policy)
    }
}

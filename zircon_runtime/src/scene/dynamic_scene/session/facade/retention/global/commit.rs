use super::super::super::super::retention;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn prune_slots(
        &mut self,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        retention::prune_slots(self, policy)
    }
}

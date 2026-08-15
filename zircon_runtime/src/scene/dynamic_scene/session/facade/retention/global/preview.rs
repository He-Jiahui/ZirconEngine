use super::super::super::super::retention;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn prepare_prune_slots(
        &self,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePrunePlan, RuntimeSessionArchiveError> {
        retention::prepare_prune_slots(self, policy)
    }

    pub fn preview_prune_slots(
        &self,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        retention::preview_prune_slots(self, policy)
    }
}

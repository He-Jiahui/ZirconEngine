use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};
use super::super::policy::policy_with_selected_protection;

impl RuntimeSessionArchive {
    pub fn preview_prune_slots_with_selected_protection(
        &self,
        policy: RuntimeSessionArchiveRetentionPolicy,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        let policy = policy_with_selected_protection(self, policy, selector)?;
        self.preview_prune_slots(policy)
    }
}

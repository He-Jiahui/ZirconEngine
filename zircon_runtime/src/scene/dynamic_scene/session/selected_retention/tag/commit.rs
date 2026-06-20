use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};
use super::super::policy::policy_with_selected_protection;

impl RuntimeSessionArchive {
    pub fn prune_slots_with_tag_and_selected_protection(
        &mut self,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        let policy = policy_with_selected_protection(self, policy, selector)?;
        self.prune_slots_with_tag(tag, policy)
    }
}

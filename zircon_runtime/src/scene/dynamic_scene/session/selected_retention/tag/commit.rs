use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn prune_slots_with_tag_and_selected_protection(
        &mut self,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        let report =
            self.preview_prune_slots_with_tag_and_selected_protection(tag, policy, selector)?;
        for slot_id in &report.removed_slot_ids {
            self.remove_slot(slot_id);
        }
        Ok(report)
    }
}

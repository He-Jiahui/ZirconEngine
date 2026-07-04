use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_prune_slots_with_tag_and_selected_protection(
        &self,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        let report = self.preview_prune_slots_with_tag(tag, policy)?;
        let selected = self.select_slot(selector)?;
        Ok(report_with_selected_tag_slot(
            self,
            tag,
            report,
            &selected.selected_slot_id,
        ))
    }
}

pub(in crate::scene::dynamic_scene::session::selected_retention) fn report_with_selected_tag_slot(
    archive: &RuntimeSessionArchive,
    tag: &str,
    mut report: RuntimeSessionArchivePruneReport,
    selected_slot_id: &str,
) -> RuntimeSessionArchivePruneReport {
    let tag = tag.trim();
    let selected_is_in_scope = archive
        .slot(selected_slot_id)
        .map(|slot| slot.metadata.tags.iter().any(|candidate| candidate == tag))
        .unwrap_or(false);
    if !selected_is_in_scope
        || !report
            .removed_slot_ids
            .iter()
            .any(|slot_id| slot_id == selected_slot_id)
    {
        return report;
    }

    report
        .removed_slot_ids
        .retain(|slot_id| slot_id != selected_slot_id);
    let mut retained_slot_ids = archive
        .slot_ids()
        .filter(|slot_id| {
            !report
                .removed_slot_ids
                .iter()
                .any(|removed| removed == slot_id)
        })
        .map(str::to_string)
        .collect::<Vec<_>>();
    retained_slot_ids.sort();
    report.retained_slot_ids = retained_slot_ids;
    report
}

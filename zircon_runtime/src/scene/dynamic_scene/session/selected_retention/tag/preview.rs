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
    report: RuntimeSessionArchivePruneReport,
    selected_slot_id: &str,
) -> RuntimeSessionArchivePruneReport {
    let tag = tag.trim();
    let selected_is_in_scope = archive
        .slot(selected_slot_id)
        .map(|slot| slot.metadata.tags.iter().any(|candidate| candidate == tag))
        .unwrap_or(false);
    if !selected_is_in_scope {
        return report;
    }

    protect_selected_slot(report, selected_slot_id)
}

fn protect_selected_slot(
    mut report: RuntimeSessionArchivePruneReport,
    selected_slot_id: &str,
) -> RuntimeSessionArchivePruneReport {
    let Ok(removed_index) = report
        .removed_slot_ids
        .binary_search_by(|slot_id| slot_id.as_str().cmp(selected_slot_id))
    else {
        return report;
    };
    report.removed_slot_ids.remove(removed_index);

    if let Err(retained_index) = report
        .retained_slot_ids
        .binary_search_by(|slot_id| slot_id.as_str().cmp(selected_slot_id))
    {
        report
            .retained_slot_ids
            .insert(retained_index, selected_slot_id.to_owned());
    }
    report
}

#[cfg(test)]
mod tests {
    use super::protect_selected_slot;
    use crate::scene::dynamic_scene::session::RuntimeSessionArchivePruneReport;

    fn report() -> RuntimeSessionArchivePruneReport {
        RuntimeSessionArchivePruneReport {
            retained_slot_ids: ["autosave", "manual-new"].map(str::to_owned).into(),
            removed_slot_ids: ["manual-mid", "manual-old"].map(str::to_owned).into(),
        }
    }

    #[test]
    fn runtime52_batch_incremental_selected_protection_preserves_canonical_order() {
        let report = protect_selected_slot(report(), "manual-mid");

        assert_eq!(
            report.retained_slot_ids,
            ["autosave", "manual-mid", "manual-new"]
        );
        assert_eq!(report.removed_slot_ids, ["manual-old"]);
    }

    #[test]
    fn runtime52_batch_incremental_selected_protection_is_noop_when_not_removed() {
        let expected = report();

        assert_eq!(
            protect_selected_slot(expected.clone(), "autosave"),
            expected
        );
    }

    #[test]
    fn runtime52_batch_incremental_selected_protection_keeps_partition_unique() {
        let mut report = report();
        report.retained_slot_ids.insert(1, "manual-mid".to_owned());

        let report = protect_selected_slot(report, "manual-mid");

        assert_eq!(
            report
                .retained_slot_ids
                .iter()
                .filter(|slot_id| slot_id.as_str() == "manual-mid")
                .count(),
            1
        );
        assert!(!report
            .removed_slot_ids
            .iter()
            .any(|slot_id| slot_id == "manual-mid"));
    }
}

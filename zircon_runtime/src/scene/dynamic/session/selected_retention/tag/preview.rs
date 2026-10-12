use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 先按标签进行普通裁剪，再额外保留选择器命中的桶内槽位。
    /// 因此可在普通保留数量之外多保留该槽位；桶外选择不改变报告，选择器未命中仍返回错误。
    /// 预览不修改档案，也不绑定后续提交。
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

// 只修改同一档案上标签预览形成的分区；桶外选择保留原报告，避免占用其他标签桶的保留名额。
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

// 报告两分区必须按槽位 ID 递增；retention 规划层提供此顺序，修订保护后仍保持有序且互斥。
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

// 固定二分保护修订的顺序与唯一性契约，防止标签预览产生重复保留项或失去规范顺序。
#[cfg(test)]
#[path = "tests/preview.rs"]
mod tests;

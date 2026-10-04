use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 先按标签进行普通裁剪，再额外保留选择器命中的桶内槽位。
    /// 因此可在普通保留数量之外多保留该槽位；桶外选择不改变报告，选择器未命中仍返回错误。
    /// 先完成所有预检再删除报告中的槽位；此入口只修改内存档案。
    pub fn prune_slots_with_tag_and_selected_protection(
        &mut self,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        let report =
            self.preview_prune_slots_with_tag_and_selected_protection(tag, policy, selector)?;
        // TODO: [CR-SESSION-SLOT-0001] 此处每删一项会推进一次修订；普通裁剪批量发布一次。
        // 需确认本入口多项删除的修订契约，并补至少两项删除的对照验证。
        for slot_id in &report.removed_slot_ids {
            self.remove_slot(slot_id);
        }
        Ok(report)
    }
}

use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy,
    RuntimeSessionSlotSelector,
};

// 将选择意图转换为显式保护 ID，供全档案裁剪共用；保持传入的其他保护项及数量目标。
pub(in crate::scene::dynamic_scene::session::selected_retention) fn policy_with_selected_protection(
    archive: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveRetentionPolicy,
    selector: RuntimeSessionSlotSelector,
) -> Result<RuntimeSessionArchiveRetentionPolicy, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    Ok(policy.with_protected_slot(report.selected_slot_id))
}

use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy,
    RuntimeSessionSlotSelector,
};

pub(in crate::scene::dynamic_scene::session::selected_retention) fn policy_with_selected_protection(
    archive: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveRetentionPolicy,
    selector: RuntimeSessionSlotSelector,
) -> Result<RuntimeSessionArchiveRetentionPolicy, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    Ok(policy.with_protected_slot(report.selected_slot_id))
}

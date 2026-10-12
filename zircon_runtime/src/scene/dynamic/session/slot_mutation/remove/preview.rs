use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotMutationPreviewReport,
};
use super::super::report::slot_mutation_report;

// 删除前的严格只读预检：档案必须有效且 ID 必须存在；返回被移除槽位的摘要。
pub(in crate::scene::dynamic_scene::session) fn preview_remove_slot(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;

    let slot = archive
        .slot(slot_id)
        .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
            slot_id: slot_id.to_string(),
        })?;

    Ok(slot_mutation_report(slot, None))
}

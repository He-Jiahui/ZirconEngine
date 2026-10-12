use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::preview::preview_rename_slot;

// 先完成目标冲突预检，再通过档案改名边界同时更新槽位 ID 与所有派生索引。
// 预览产生的目标 ID 与源索引仅在本次独占借用内使用，不能保存为跨调用的提交凭据。
pub(in crate::scene::dynamic_scene::session) fn rename_slot(
    archive: &mut RuntimeSessionArchive,
    old_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = preview_rename_slot(archive, old_slot_id, new_slot_id)?;
    let new_slot_id = report
        .destination_slot_id
        .expect("rename slot preview always reports a destination slot id");
    let slot_index = archive
        .indexed_slot_index(&report.source_slot_id)
        .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
            slot_id: report.source_slot_id.clone(),
        })?;

    archive.commit_slot_rename(&report.source_slot_id, slot_index, new_slot_id)
}

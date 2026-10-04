use super::super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::metadata::import_slot_from_archive_with_metadata;

// 提交跨档案导入并继承源槽位全部元数据；不因复制自动刷新时间或标签。
// 显式源 ID 按原值查询，目标新 ID 由共同预检修剪并检查冲突。
pub(in crate::scene::dynamic_scene::session) fn import_slot_from_archive(
    target: &mut RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<(), RuntimeSessionArchiveError> {
    let metadata = incoming.require_slot(source_slot_id)?.metadata.clone();
    import_slot_from_archive_with_metadata(target, incoming, source_slot_id, new_slot_id, metadata)
}

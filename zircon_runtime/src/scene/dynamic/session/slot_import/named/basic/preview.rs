use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
};
use super::super::metadata::preview_import_slot_from_archive_with_metadata;

// 只读预览跨档案导入并继承源槽位全部元数据；不因复制自动刷新时间或标签。
// 显式源 ID 按原值查询，目标新 ID 由共同预检修剪并检查冲突。
pub(in crate::scene::dynamic_scene::session) fn preview_import_slot_from_archive(
    target: &RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    let metadata = incoming.require_slot(source_slot_id)?.metadata.clone();
    preview_import_slot_from_archive_with_metadata(
        target,
        incoming,
        source_slot_id,
        new_slot_id,
        metadata,
    )
}

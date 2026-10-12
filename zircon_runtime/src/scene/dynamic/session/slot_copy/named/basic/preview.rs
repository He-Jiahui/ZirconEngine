use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
};
use super::super::metadata::preview_copy_slot_with_metadata;

// 只读预览同档案复制并继承源槽位全部元数据；不因复制自动刷新时间或标签。
// 显式源 ID 按原值查询，目标新 ID 由共同预检修剪并检查冲突。
pub(in crate::scene::dynamic_scene::session) fn preview_copy_slot(
    archive: &RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    let metadata = archive.require_slot(source_slot_id)?.metadata.clone();
    preview_copy_slot_with_metadata(archive, source_slot_id, new_slot_id, metadata)
}

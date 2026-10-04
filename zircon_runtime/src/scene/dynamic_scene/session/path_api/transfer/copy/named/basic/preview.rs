use std::path::Path;

use super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotImportPreviewReport,
};

impl RuntimeSessionArchive {
    /// 载入已有档案，按原值查找源槽位后预览复制摘要，继承全部源元数据且不刷新时间。
    /// 新 ID 修剪后须非空且未占用；报告不绑定后续提交。
    pub fn preview_copy_slot_from_path(
        path: impl AsRef<Path>,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        path_transfer::preview_copy_slot_from_path(path, source_slot_id, new_slot_id)
    }
}

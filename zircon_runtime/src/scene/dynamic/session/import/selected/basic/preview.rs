use std::path::Path;

use super::super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 预检磁盘来源向已有目标的单槽导入；路径相等或双方规范路径相等时拒绝。
    /// 新 ID 修剪后须非空且未占用。
    /// 选择器在来源快照中解析；继承源槽位元数据，不自动刷新更新时间。
    /// 预览读取双方当次内容，不写盘，也不锁定后续提交的选择结果。
    pub fn preview_import_selected_slot_from_archive_path_at_path(
        path: impl AsRef<Path>,
        source_path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        path_transfer::preview_import_selected_slot_from_archive_path_at_path(
            path,
            source_path,
            selector,
            new_slot_id,
        )
    }
}

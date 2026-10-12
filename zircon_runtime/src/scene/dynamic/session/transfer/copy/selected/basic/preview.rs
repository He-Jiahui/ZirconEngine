use std::path::Path;

use super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 在本次载入的档案上解析选择器后预览复制摘要，继承全部源元数据且不刷新时间。
    /// 新 ID 修剪后须非空且未占用；报告不绑定后续提交。
    pub fn preview_copy_selected_slot_from_path(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        path_transfer::preview_copy_selected_slot_from_path(path, selector, new_slot_id)
    }
}

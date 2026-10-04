use super::super::super::super::super::super::{
    slot_copy, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 在当前档案解析选择器后预览复制摘要，继承全部源元数据且不刷新时间。
    /// 新 ID 修剪后须非空且未占用；报告不绑定后续提交。
    pub fn preview_copy_selected_slot(
        &self,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        slot_copy::preview_copy_selected_slot(self, selector, new_slot_id)
    }
}

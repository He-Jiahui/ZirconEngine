use super::super::super::super::super::super::{
    slot_copy, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 在当前档案解析选择器后预览复制摘要，整体采用规范化后的替换元数据。
    /// 新 ID 修剪后须非空且未占用；报告不绑定后续提交。
    pub fn preview_copy_selected_slot_with_metadata(
        &self,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        slot_copy::preview_copy_selected_slot_with_metadata(self, selector, new_slot_id, metadata)
    }
}

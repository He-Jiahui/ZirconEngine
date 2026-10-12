use super::super::super::super::super::super::{
    slot_copy, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotImportPreviewReport,
};

impl RuntimeSessionArchive {
    /// 按原值查找源槽位后预览复制摘要，继承全部源元数据且不刷新时间。
    /// 新 ID 修剪后须非空且未占用；报告不绑定后续提交。
    pub fn preview_copy_slot(
        &self,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        slot_copy::preview_copy_slot(self, source_slot_id, new_slot_id)
    }
}

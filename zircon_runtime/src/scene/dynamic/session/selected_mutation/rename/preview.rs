use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotMutationPreviewReport,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 只读预览选中槽位改名；新 ID 修剪后不得与其他槽位冲突。
    /// 选择器必须命中当前档案；此报告不绑定后续提交的目标，提交时会重新解析选择器。
    pub fn preview_rename_selected_slot(
        &self,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_rename_slot(&report.selected_slot_id, new_slot_id)
    }
}

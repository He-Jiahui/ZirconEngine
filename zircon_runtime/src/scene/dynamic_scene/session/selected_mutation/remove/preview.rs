use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotMutationPreviewReport,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 只读预览选中槽位的删除，返回其身份、元数据和场景规模。
    /// 选择器必须命中当前档案；此报告不绑定后续提交的目标，提交时会重新解析选择器。
    pub fn preview_remove_selected_slot(
        &self,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_remove_slot(&report.selected_slot_id)
    }
}

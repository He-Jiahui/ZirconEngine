use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotMutationPreviewReport,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 只读预览选中槽位更新时间的替换；时间由调用者提供。
    /// 选择器必须命中当前档案；此报告不绑定后续提交的目标，提交时会重新解析选择器。
    pub fn preview_touch_selected_slot(
        &self,
        selector: RuntimeSessionSlotSelector,
        updated_at_unix_millis: u64,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_touch_slot(&report.selected_slot_id, updated_at_unix_millis)
    }
}

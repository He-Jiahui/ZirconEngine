use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotMutationPreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 只读预览选中槽位的元数据整体替换，返回规范化后的新元数据。
    /// 选择器必须命中当前档案；此报告不绑定后续提交的目标，提交时会重新解析选择器。
    pub fn preview_update_selected_slot_metadata(
        &self,
        selector: RuntimeSessionSlotSelector,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_update_slot_metadata(&report.selected_slot_id, metadata)
    }
}

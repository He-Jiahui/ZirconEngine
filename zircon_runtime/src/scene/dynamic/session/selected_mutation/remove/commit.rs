use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 删除当前选中槽位并返回完整槽位，供调用者决定是否保留或转移该快照。
    /// 选择器必须命中当前档案；此调用只变更内存，路径 API 才负责保存档案。
    pub fn remove_selected_slot(
        &mut self,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionSlot, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.remove_slot(&report.selected_slot_id).ok_or_else(|| {
            RuntimeSessionArchiveError::MissingSlot {
                slot_id: report.selected_slot_id,
            }
        })
    }
}

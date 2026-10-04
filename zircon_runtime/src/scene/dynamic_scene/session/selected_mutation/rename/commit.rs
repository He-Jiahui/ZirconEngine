use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 改名当前选中槽位，保留其元数据和场景；新 ID 修剪后不得与其他槽位冲突。
    /// 选择器必须命中当前档案；此调用只变更内存，路径 API 才负责保存档案。
    pub fn rename_selected_slot(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.rename_slot(&report.selected_slot_id, new_slot_id)
    }
}

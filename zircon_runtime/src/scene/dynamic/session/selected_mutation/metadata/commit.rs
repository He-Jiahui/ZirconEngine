use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 整体替换当前选中槽位的会话元数据，保留其场景负载。
    /// 选择器必须命中当前档案；此调用只变更内存，路径 API 才负责保存档案。
    pub fn update_selected_slot_metadata(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.update_slot_metadata(&report.selected_slot_id, metadata)
    }
}

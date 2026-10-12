use super::super::super::super::super::super::{
    slot_copy, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 在当前档案解析选择器后复制到新 ID，整体采用规范化后的替换元数据；保留源槽位。
    /// 新 ID 修剪后须非空且未占用。
    pub fn copy_selected_slot_with_metadata(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_copy::copy_selected_slot_with_metadata(self, selector, new_slot_id, metadata)
    }
}

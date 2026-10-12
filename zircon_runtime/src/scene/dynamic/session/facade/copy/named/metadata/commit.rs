use super::super::super::super::super::super::{
    slot_copy, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
};

impl RuntimeSessionArchive {
    /// 按原值查找源槽位后复制到新 ID，整体采用规范化后的替换元数据；保留源槽位。
    /// 新 ID 修剪后须非空且未占用。
    pub fn copy_slot_with_metadata(
        &mut self,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_copy::copy_slot_with_metadata(self, source_slot_id, new_slot_id, metadata)
    }
}

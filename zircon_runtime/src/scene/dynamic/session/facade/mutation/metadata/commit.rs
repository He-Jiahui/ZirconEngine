use super::super::super::super::slot_mutation;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 整体替换槽位元数据并刷新标签与时间索引，保留场景；缺失字段不会从旧元数据补齐。
    pub fn update_slot_metadata(
        &mut self,
        slot_id: &str,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_mutation::update_slot_metadata(self, slot_id, metadata)
    }
}

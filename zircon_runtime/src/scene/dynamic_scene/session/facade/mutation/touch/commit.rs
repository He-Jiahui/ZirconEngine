use super::super::super::super::slot_mutation;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 设置调用者给定的更新时间并刷新排序索引；保留其他元数据与场景，不要求时间递增。
    pub fn touch_slot(
        &mut self,
        slot_id: &str,
        updated_at_unix_millis: u64,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_mutation::touch_slot(self, slot_id, updated_at_unix_millis)
    }
}

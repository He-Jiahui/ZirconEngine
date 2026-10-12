use super::super::super::super::slot_mutation;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 重命名槽位并维护查询索引；目标 ID 已存在时保留原档案状态。
    pub fn rename_slot(
        &mut self,
        old_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_mutation::rename_slot(self, old_slot_id, new_slot_id)
    }
}

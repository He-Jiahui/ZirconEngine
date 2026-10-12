use super::super::super::super::super::super::{
    slot_copy, RuntimeSessionArchive, RuntimeSessionArchiveError,
};

impl RuntimeSessionArchive {
    /// 按原值查找源槽位后复制到新 ID，继承全部源元数据且不刷新时间；保留源槽位。
    /// 新 ID 修剪后须非空且未占用。
    pub fn copy_slot(
        &mut self,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_copy::copy_slot(self, source_slot_id, new_slot_id)
    }
}

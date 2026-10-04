use super::super::super::slot_mutation;
use super::super::super::*;

impl RuntimeSessionArchive {
    /// 移除并返回完整槽位，缺失时返回空值；需要档案整体校验的流程应先调用删除预览。
    pub fn remove_slot(&mut self, slot_id: &str) -> Option<RuntimeSessionSlot> {
        slot_mutation::remove_slot(self, slot_id)
    }
}

use super::super::super::slot_store;
use super::super::super::*;

impl RuntimeSessionArchive {
    /// 追加外部构造的槽位；同名槽位视为冲突，调用方应使用 upsert 明确表达替换意图。
    pub fn push_slot(
        &mut self,
        slot: RuntimeSessionSlot,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_store::push_slot(self, slot)
    }
}

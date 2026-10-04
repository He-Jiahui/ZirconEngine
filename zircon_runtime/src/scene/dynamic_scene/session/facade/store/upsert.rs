use super::super::super::slot_store;
use super::super::super::*;

impl RuntimeSessionArchive {
    /// 以槽位 ID 为身份插入或替换快照，并同步次级索引和代际。
    pub fn upsert_slot(
        &mut self,
        slot: RuntimeSessionSlot,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_store::upsert_slot(self, slot)
    }
}

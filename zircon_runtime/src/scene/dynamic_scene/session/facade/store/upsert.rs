use super::super::super::slot_store;
use super::super::super::*;

impl RuntimeSessionArchive {
    pub fn upsert_slot(
        &mut self,
        slot: RuntimeSessionSlot,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_store::upsert_slot(self, slot)
    }
}

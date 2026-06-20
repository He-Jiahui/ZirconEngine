use super::super::super::slot_store;
use super::super::super::*;

impl RuntimeSessionArchive {
    pub fn push_slot(
        &mut self,
        slot: RuntimeSessionSlot,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_store::push_slot(self, slot)
    }
}

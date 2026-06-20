use super::super::super::slot_mutation;
use super::super::super::*;

impl RuntimeSessionArchive {
    pub fn remove_slot(&mut self, slot_id: &str) -> Option<RuntimeSessionSlot> {
        slot_mutation::remove_slot(self, slot_id)
    }
}

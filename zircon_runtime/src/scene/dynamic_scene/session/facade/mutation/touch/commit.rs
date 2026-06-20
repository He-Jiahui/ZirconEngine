use super::super::super::super::slot_mutation;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn touch_slot(
        &mut self,
        slot_id: &str,
        updated_at_unix_millis: u64,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_mutation::touch_slot(self, slot_id, updated_at_unix_millis)
    }
}

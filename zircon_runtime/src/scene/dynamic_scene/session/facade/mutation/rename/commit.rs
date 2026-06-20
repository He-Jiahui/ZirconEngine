use super::super::super::super::slot_mutation;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn rename_slot(
        &mut self,
        old_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_mutation::rename_slot(self, old_slot_id, new_slot_id)
    }
}

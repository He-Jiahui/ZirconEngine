use super::super::super::super::super::super::{
    slot_copy, RuntimeSessionArchive, RuntimeSessionArchiveError,
};

impl RuntimeSessionArchive {
    pub fn copy_slot(
        &mut self,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_copy::copy_slot(self, source_slot_id, new_slot_id)
    }
}

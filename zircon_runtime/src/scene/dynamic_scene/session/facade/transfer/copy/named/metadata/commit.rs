use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata, slot_copy,
};

impl RuntimeSessionArchive {
    pub fn copy_slot_with_metadata(
        &mut self,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_copy::copy_slot_with_metadata(self, source_slot_id, new_slot_id, metadata)
    }
}

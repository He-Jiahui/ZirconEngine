use super::super::super::super::slot_mutation;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn update_slot_metadata(
        &mut self,
        slot_id: &str,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_mutation::update_slot_metadata(self, slot_id, metadata)
    }
}

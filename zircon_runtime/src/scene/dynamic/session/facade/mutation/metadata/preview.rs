use super::super::super::super::slot_mutation;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn preview_update_slot_metadata(
        &self,
        slot_id: &str,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        slot_mutation::preview_update_slot_metadata(self, slot_id, metadata)
    }
}

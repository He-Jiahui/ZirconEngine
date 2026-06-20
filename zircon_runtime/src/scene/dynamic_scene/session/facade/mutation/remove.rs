use super::super::super::slot_mutation;
use super::super::super::*;

impl RuntimeSessionArchive {
    pub fn preview_remove_slot(
        &self,
        slot_id: &str,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        slot_mutation::preview_remove_slot(self, slot_id)
    }
}

use super::super::super::super::slot_mutation;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn preview_rename_slot(
        &self,
        old_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        slot_mutation::preview_rename_slot(self, old_slot_id, new_slot_id)
    }
}

use super::super::super::super::slot_mutation;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn preview_touch_slot(
        &self,
        slot_id: &str,
        updated_at_unix_millis: u64,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        slot_mutation::preview_touch_slot(self, slot_id, updated_at_unix_millis)
    }
}

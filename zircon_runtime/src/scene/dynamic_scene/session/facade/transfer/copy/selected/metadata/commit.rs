use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector, slot_copy,
};

impl RuntimeSessionArchive {
    pub fn copy_selected_slot_with_metadata(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_copy::copy_selected_slot_with_metadata(self, selector, new_slot_id, metadata)
    }
}

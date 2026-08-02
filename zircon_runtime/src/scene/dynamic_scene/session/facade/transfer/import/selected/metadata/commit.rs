use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector, slot_import,
};

impl RuntimeSessionArchive {
    pub fn import_selected_slot_from_archive_with_metadata(
        &mut self,
        incoming: &RuntimeSessionArchive,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_import::import_selected_slot_from_archive_with_metadata(
            self,
            incoming,
            selector,
            new_slot_id,
            metadata,
        )
    }
}

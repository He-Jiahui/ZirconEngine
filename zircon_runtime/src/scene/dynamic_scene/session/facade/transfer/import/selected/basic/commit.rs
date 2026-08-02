use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector, slot_import,
};

impl RuntimeSessionArchive {
    pub fn import_selected_slot_from_archive(
        &mut self,
        incoming: &RuntimeSessionArchive,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_import::import_selected_slot_from_archive(self, incoming, selector, new_slot_id)
    }
}

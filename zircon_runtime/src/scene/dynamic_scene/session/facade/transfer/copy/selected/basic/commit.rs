use super::super::super::super::super::super::{
    slot_copy, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn copy_selected_slot(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_copy::copy_selected_slot(self, selector, new_slot_id)
    }
}

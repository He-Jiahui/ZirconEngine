use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector, slot_copy,
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

use super::super::super::super::super::{
    slot_export, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn selected_single_slot_archive(
        &self,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        slot_export::selected_single_slot_archive(self, selector)
    }
}

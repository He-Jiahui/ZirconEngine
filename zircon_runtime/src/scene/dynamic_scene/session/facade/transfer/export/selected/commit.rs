use super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector, slot_export,
};

impl RuntimeSessionArchive {
    pub fn selected_single_slot_archive(
        &self,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        slot_export::selected_single_slot_archive(self, selector)
    }
}

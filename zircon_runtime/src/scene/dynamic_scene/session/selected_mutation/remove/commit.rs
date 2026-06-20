use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn remove_selected_slot(
        &mut self,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionSlot, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.remove_slot(&report.selected_slot_id).ok_or_else(|| {
            RuntimeSessionArchiveError::MissingSlot {
                slot_id: report.selected_slot_id,
            }
        })
    }
}

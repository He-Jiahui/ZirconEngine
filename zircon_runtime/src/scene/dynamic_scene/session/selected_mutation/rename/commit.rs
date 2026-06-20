use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn rename_selected_slot(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.rename_slot(&report.selected_slot_id, new_slot_id)
    }
}

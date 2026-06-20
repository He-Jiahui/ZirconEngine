use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn touch_selected_slot(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        updated_at_unix_millis: u64,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.touch_slot(&report.selected_slot_id, updated_at_unix_millis)
    }
}

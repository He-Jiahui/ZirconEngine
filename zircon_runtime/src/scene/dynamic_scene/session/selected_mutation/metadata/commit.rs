use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn update_selected_slot_metadata(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.update_slot_metadata(&report.selected_slot_id, metadata)
    }
}

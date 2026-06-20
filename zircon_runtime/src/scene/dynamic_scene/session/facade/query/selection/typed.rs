use super::super::super::super::query as session_query;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn select_slot(
        &self,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionSlotSelectionReport, RuntimeSessionArchiveError> {
        session_query::select_slot(self, selector)
    }
}

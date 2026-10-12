use super::super::super::super::query as session_query;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn latest_updated_slot_id(&self) -> Result<Option<String>, RuntimeSessionArchiveError> {
        session_query::latest_updated_slot_id(self)
    }

    pub fn oldest_updated_slot_id(&self) -> Result<Option<String>, RuntimeSessionArchiveError> {
        session_query::oldest_updated_slot_id(self)
    }
}

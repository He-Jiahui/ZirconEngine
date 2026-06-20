use super::super::super::super::query as session_query;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn latest_updated_slot_id_with_tag(
        &self,
        tag: &str,
    ) -> Result<Option<String>, RuntimeSessionArchiveError> {
        session_query::latest_updated_slot_id_with_tag(self, tag)
    }

    pub fn oldest_updated_slot_id_with_tag(
        &self,
        tag: &str,
    ) -> Result<Option<String>, RuntimeSessionArchiveError> {
        session_query::oldest_updated_slot_id_with_tag(self, tag)
    }
}

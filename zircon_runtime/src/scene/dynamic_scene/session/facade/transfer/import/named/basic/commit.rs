use super::super::super::super::super::super::{
    slot_import, RuntimeSessionArchive, RuntimeSessionArchiveError,
};

impl RuntimeSessionArchive {
    pub fn import_slot_from_archive(
        &mut self,
        incoming: &RuntimeSessionArchive,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_import::import_slot_from_archive(self, incoming, source_slot_id, new_slot_id)
    }
}

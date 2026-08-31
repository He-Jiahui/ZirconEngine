use super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, slot_export,
};

impl RuntimeSessionArchive {
    pub fn single_slot_archive(&self, slot_id: &str) -> Result<Self, RuntimeSessionArchiveError> {
        slot_export::single_slot_archive(self, slot_id)
    }
}

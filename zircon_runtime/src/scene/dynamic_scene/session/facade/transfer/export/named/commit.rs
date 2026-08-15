use super::super::super::super::super::{
    slot_export, RuntimeSessionArchive, RuntimeSessionArchiveError,
};

impl RuntimeSessionArchive {
    pub fn single_slot_archive(&self, slot_id: &str) -> Result<Self, RuntimeSessionArchiveError> {
        slot_export::single_slot_archive(self, slot_id)
    }
}

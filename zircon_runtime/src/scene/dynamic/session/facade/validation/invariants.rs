use super::super::super::validation;
use super::super::super::*;

impl RuntimeSessionArchive {
    pub(in crate::scene::dynamic_scene::session) fn require_slot(
        &self,
        slot_id: &str,
    ) -> Result<&RuntimeSessionSlot, RuntimeSessionArchiveError> {
        validation::require_slot(self, slot_id)
    }

    pub(in crate::scene::dynamic_scene::session) fn normalize_slot_metadata(&mut self) {
        validation::normalize_slot_metadata(self);
    }
}

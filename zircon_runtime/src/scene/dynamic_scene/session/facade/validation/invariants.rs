use super::super::super::validation;
use super::super::super::*;

impl RuntimeSessionArchive {
    pub(in crate::scene::dynamic_scene::session) fn require_slot(
        &self,
        slot_id: &str,
    ) -> Result<&RuntimeSessionSlot, RuntimeSessionArchiveError> {
        validation::require_slot(self, slot_id)
    }

    // Keep mutable slot access private so callers cannot bypass id sorting,
    // duplicate checks, or metadata normalization.
    pub(in crate::scene::dynamic_scene::session) fn slot_mut(
        &mut self,
        slot_id: &str,
    ) -> Option<&mut RuntimeSessionSlot> {
        validation::slot_mut(self, slot_id)
    }

    pub(in crate::scene::dynamic_scene::session) fn sort_slots(&mut self) {
        validation::sort_slots(self);
    }

    pub(in crate::scene::dynamic_scene::session) fn normalize_slot_metadata(&mut self) {
        validation::normalize_slot_metadata(self);
    }
}

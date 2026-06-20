use crate::scene::LevelSystem;

use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn capture_level_selected_slot(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.capture_level_slot(report.selected_slot_id, level)
    }

    pub fn capture_level_selected_slot_preserving_metadata(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        let world = level.snapshot();
        self.capture_world_slot(report.selected_slot_id, &world, report.summary.metadata)
    }
}

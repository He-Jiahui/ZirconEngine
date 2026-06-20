use crate::scene::World;

use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn capture_world_selected_slot(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.capture_world_slot(report.selected_slot_id, world, metadata)
    }

    pub fn capture_world_selected_slot_preserving_metadata(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        world: &World,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.capture_world_slot(report.selected_slot_id, world, report.summary.metadata)
    }
}

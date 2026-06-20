use std::path::Path;

use crate::scene::{LevelSystem, World};

use super::super::super::super::{
    path_restore, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotDiffReport,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn diff_selected_slot_from_path_with_world(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        world: &World,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        path_restore::diff_selected_slot_from_path_with_world(path, selector, world)
    }

    pub fn diff_selected_slot_from_path_with_level(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        path_restore::diff_selected_slot_from_path_with_level(path, selector, level)
    }
}

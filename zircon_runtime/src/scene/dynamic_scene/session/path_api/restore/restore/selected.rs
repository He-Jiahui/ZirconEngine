use std::path::Path;

use crate::scene::{LevelSystem, World};

use super::super::super::super::{
    path_restore, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionLevelRestoreReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn restore_selected_slot_from_path_to_empty_world(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<World, RuntimeSessionArchiveError> {
        path_restore::restore_selected_slot_from_path_to_empty_world(path, selector)
    }

    pub fn restore_selected_slot_from_path_into_level(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionLevelRestoreReport, RuntimeSessionArchiveError> {
        path_restore::restore_selected_slot_from_path_into_level(path, selector, level)
    }
}

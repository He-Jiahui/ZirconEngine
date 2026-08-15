use std::path::Path;

use crate::scene::{LevelSystem, World};

use super::super::super::super::{
    path_restore, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionLevelRestoreReport,
};

impl RuntimeSessionArchive {
    pub fn restore_slot_from_path_to_empty_world(
        path: impl AsRef<Path>,
        slot_id: &str,
    ) -> Result<World, RuntimeSessionArchiveError> {
        path_restore::restore_slot_from_path_to_empty_world(path, slot_id)
    }

    pub fn restore_slot_from_path_into_level(
        path: impl AsRef<Path>,
        slot_id: &str,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionLevelRestoreReport, RuntimeSessionArchiveError> {
        path_restore::restore_slot_from_path_into_level(path, slot_id, level)
    }
}

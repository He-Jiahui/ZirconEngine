use std::path::Path;

use crate::scene::{LevelSystem, World};

use super::super::super::super::{
    path_restore, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotDiffReport,
};

impl RuntimeSessionArchive {
    pub fn diff_slot_from_path_with_world(
        path: impl AsRef<Path>,
        slot_id: &str,
        world: &World,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        path_restore::diff_slot_from_path_with_world(path, slot_id, world)
    }

    pub fn diff_slot_from_path_with_level(
        path: impl AsRef<Path>,
        slot_id: &str,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        path_restore::diff_slot_from_path_with_level(path, slot_id, level)
    }
}

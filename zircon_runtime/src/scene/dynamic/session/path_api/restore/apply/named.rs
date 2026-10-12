use std::path::Path;

use crate::scene::{LevelSystem, World};

use super::super::super::super::super::EntityRemap;
use super::super::super::super::{path_restore, RuntimeSessionArchive, RuntimeSessionArchiveError};

impl RuntimeSessionArchive {
    pub fn apply_slot_from_path_to_world(
        path: impl AsRef<Path>,
        slot_id: &str,
        world: &mut World,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        path_restore::apply_slot_from_path_to_world(path, slot_id, world)
    }

    pub fn apply_slot_from_path_to_level(
        path: impl AsRef<Path>,
        slot_id: &str,
        level: &LevelSystem,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        path_restore::apply_slot_from_path_to_level(path, slot_id, level)
    }
}

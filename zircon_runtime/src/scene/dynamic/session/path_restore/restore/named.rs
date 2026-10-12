use std::path::Path;

use crate::scene::{LevelSystem, World};

use super::super::super::{io, RuntimeSessionArchiveError, RuntimeSessionLevelRestoreReport};

pub(in crate::scene::dynamic_scene::session) fn restore_slot_from_path_to_empty_world(
    path: impl AsRef<Path>,
    slot_id: &str,
) -> Result<World, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.restore_slot_to_empty_world(slot_id)
}

pub(in crate::scene::dynamic_scene::session) fn restore_slot_from_path_into_level(
    path: impl AsRef<Path>,
    slot_id: &str,
    level: &LevelSystem,
) -> Result<RuntimeSessionLevelRestoreReport, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.restore_slot_into_level(slot_id, level)
}

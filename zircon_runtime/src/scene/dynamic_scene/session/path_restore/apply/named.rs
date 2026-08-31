use std::path::Path;

use crate::scene::{EntityRemap, LevelSystem, World};

use super::super::super::{RuntimeSessionArchiveError, io};

pub(in crate::scene::dynamic_scene::session) fn apply_slot_from_path_to_world(
    path: impl AsRef<Path>,
    slot_id: &str,
    world: &mut World,
) -> Result<EntityRemap, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.apply_slot(slot_id, world)
}

pub(in crate::scene::dynamic_scene::session) fn apply_slot_from_path_to_level(
    path: impl AsRef<Path>,
    slot_id: &str,
    level: &LevelSystem,
) -> Result<EntityRemap, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.apply_slot_to_level(slot_id, level)
}

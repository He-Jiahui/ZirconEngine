use std::path::Path;

use crate::scene::{LevelSystem, World};

use super::super::super::{RuntimeSessionArchiveError, RuntimeSessionSlotDiffReport, io};

pub(in crate::scene::dynamic_scene::session) fn diff_slot_from_path_with_world(
    path: impl AsRef<Path>,
    slot_id: &str,
    world: &World,
) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.diff_slot_with_world(slot_id, world)
}

pub(in crate::scene::dynamic_scene::session) fn diff_slot_from_path_with_level(
    path: impl AsRef<Path>,
    slot_id: &str,
    level: &LevelSystem,
) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.diff_slot_with_level(slot_id, level)
}

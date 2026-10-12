use std::path::Path;

use crate::scene::{EntityRemap, LevelSystem, World};

use super::super::super::{io, RuntimeSessionArchiveError, RuntimeSessionSlotSelector};

pub(in crate::scene::dynamic_scene::session) fn apply_selected_slot_from_path_to_world(
    path: impl AsRef<Path>,
    selector: RuntimeSessionSlotSelector,
    world: &mut World,
) -> Result<EntityRemap, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.apply_selected_slot(selector, world)
}

pub(in crate::scene::dynamic_scene::session) fn apply_selected_slot_from_path_to_level(
    path: impl AsRef<Path>,
    selector: RuntimeSessionSlotSelector,
    level: &LevelSystem,
) -> Result<EntityRemap, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.apply_selected_slot_to_level(selector, level)
}

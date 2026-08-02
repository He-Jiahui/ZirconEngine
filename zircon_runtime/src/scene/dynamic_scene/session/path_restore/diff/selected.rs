use std::path::Path;

use crate::scene::{LevelSystem, World};

use super::super::super::{
    RuntimeSessionArchiveError, RuntimeSessionSlotDiffReport, RuntimeSessionSlotSelector, io,
};

pub(in crate::scene::dynamic_scene::session) fn diff_selected_slot_from_path_with_world(
    path: impl AsRef<Path>,
    selector: RuntimeSessionSlotSelector,
    world: &World,
) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.diff_selected_slot_with_world(selector, world)
}

pub(in crate::scene::dynamic_scene::session) fn diff_selected_slot_from_path_with_level(
    path: impl AsRef<Path>,
    selector: RuntimeSessionSlotSelector,
    level: &LevelSystem,
) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.diff_selected_slot_with_level(selector, level)
}

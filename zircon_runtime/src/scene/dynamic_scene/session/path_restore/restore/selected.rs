use std::path::Path;

use crate::scene::{LevelSystem, World};

use super::super::super::{
    RuntimeSessionArchiveError, RuntimeSessionLevelRestoreReport, RuntimeSessionSlotSelector, io,
};

pub(in crate::scene::dynamic_scene::session) fn restore_selected_slot_from_path_to_empty_world(
    path: impl AsRef<Path>,
    selector: RuntimeSessionSlotSelector,
) -> Result<World, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.restore_selected_slot_to_empty_world(selector)
}

pub(in crate::scene::dynamic_scene::session) fn restore_selected_slot_from_path_into_level(
    path: impl AsRef<Path>,
    selector: RuntimeSessionSlotSelector,
    level: &LevelSystem,
) -> Result<RuntimeSessionLevelRestoreReport, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.restore_selected_slot_into_level(selector, level)
}

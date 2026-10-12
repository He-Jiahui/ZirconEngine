use crate::scene::{LevelSystem, World};

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionLevelRestoreReport,
};

pub(in crate::scene::dynamic_scene::session) fn restore_slot_to_empty_world(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
) -> Result<World, RuntimeSessionArchiveError> {
    archive.require_slot(slot_id)?.restore_to_empty_world()
}

pub(in crate::scene::dynamic_scene::session) fn restore_slot_into_level(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
    level: &LevelSystem,
) -> Result<RuntimeSessionLevelRestoreReport, RuntimeSessionArchiveError> {
    archive.require_slot(slot_id)?.restore_into_level(level)
}

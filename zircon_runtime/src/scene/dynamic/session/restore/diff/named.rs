use crate::scene::{LevelSystem, World};

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotDiffReport,
};

pub(in crate::scene::dynamic_scene::session) fn diff_slot_with_world(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
    world: &World,
) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
    archive.require_slot(slot_id)?.diff_world(world)
}

pub(in crate::scene::dynamic_scene::session) fn diff_slot_with_level(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
    level: &LevelSystem,
) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
    archive.require_slot(slot_id)?.diff_level(level)
}

use crate::scene::{EntityRemap, LevelSystem, World};

use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};

pub(in crate::scene::dynamic_scene::session) fn apply_slot(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
    world: &mut World,
) -> Result<EntityRemap, RuntimeSessionArchiveError> {
    archive.require_slot(slot_id)?.apply_to_world(world)
}

pub(in crate::scene::dynamic_scene::session) fn apply_slot_to_level(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
    level: &LevelSystem,
) -> Result<EntityRemap, RuntimeSessionArchiveError> {
    archive.require_slot(slot_id)?.apply_to_level(level)
}

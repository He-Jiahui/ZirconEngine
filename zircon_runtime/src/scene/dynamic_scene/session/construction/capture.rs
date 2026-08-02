use crate::scene::{LevelSystem, World};

use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata, RuntimeSessionSlot,
};
use super::archive::from_slots;

pub(in crate::scene::dynamic_scene::session) fn from_world(
    slot_id: impl Into<String>,
    world: &World,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    from_world_with_metadata(slot_id, world, RuntimeSessionMetadata::default())
}

pub(in crate::scene::dynamic_scene::session) fn from_world_with_metadata(
    slot_id: impl Into<String>,
    world: &World,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    let archive = from_slots(vec![RuntimeSessionSlot::from_world_with_metadata(
        slot_id, world, metadata,
    )?])?;
    archive.record_capture();
    Ok(archive)
}

pub(in crate::scene::dynamic_scene::session) fn from_level(
    slot_id: impl Into<String>,
    level: &LevelSystem,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    let archive = from_slots(vec![RuntimeSessionSlot::from_level(slot_id, level)?])?;
    archive.record_capture();
    Ok(archive)
}

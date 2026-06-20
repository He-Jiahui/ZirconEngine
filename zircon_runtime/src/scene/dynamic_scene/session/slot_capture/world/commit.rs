use crate::scene::World;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
};
use super::preview::preview_world_slot;

pub(in crate::scene::dynamic_scene::session) fn capture_world_slot(
    archive: &mut RuntimeSessionArchive,
    slot_id: impl Into<String>,
    world: &World,
    metadata: RuntimeSessionMetadata,
) -> Result<(), RuntimeSessionArchiveError> {
    let preview = preview_world_slot(archive, slot_id, world, metadata)?;
    archive.upsert_slot(preview.slot)
}

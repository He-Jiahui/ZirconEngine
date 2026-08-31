use crate::scene::World;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata, RuntimeSessionSlot,
};
use super::super::preview::{RuntimeSessionSlotCapturePreview, capture_preview};

pub(in crate::scene::dynamic_scene::session) fn preview_world_slot(
    archive: &RuntimeSessionArchive,
    slot_id: impl Into<String>,
    world: &World,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionSlotCapturePreview, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    capture_preview(
        archive,
        RuntimeSessionSlot::from_world_with_metadata(slot_id, world, metadata)?,
    )
}

use crate::scene::World;

use super::super::super::{
    slot_capture, RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata,
};
use super::super::apply::prepare_capture_preview_with_retention;

pub(in crate::scene::dynamic_scene::session) fn preview_world_slot_with_retention(
    archive: &RuntimeSessionArchive,
    slot_id: impl Into<String>,
    world: &World,
    metadata: RuntimeSessionMetadata,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
    let preview = slot_capture::preview_world_slot(archive, slot_id, world, metadata)?;
    Ok(prepare_capture_preview_with_retention(archive, preview, None, policy)?.report(archive))
}

pub(in crate::scene::dynamic_scene::session) fn preview_world_slot_with_tag_retention(
    archive: &RuntimeSessionArchive,
    tag: &str,
    slot_id: impl Into<String>,
    world: &World,
    metadata: RuntimeSessionMetadata,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
    let preview = slot_capture::preview_world_slot(archive, slot_id, world, metadata)?;
    Ok(
        prepare_capture_preview_with_retention(archive, preview, Some(tag), policy)?
            .report(archive),
    )
}

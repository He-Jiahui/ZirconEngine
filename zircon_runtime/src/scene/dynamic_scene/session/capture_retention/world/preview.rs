use crate::scene::World;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata, slot_capture,
};
use super::super::apply::apply_capture_preview_with_retention;

pub(in crate::scene::dynamic_scene::session) fn preview_world_slot_with_retention(
    archive: &RuntimeSessionArchive,
    slot_id: impl Into<String>,
    world: &World,
    metadata: RuntimeSessionMetadata,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
    let preview = slot_capture::preview_world_slot(archive, slot_id, world, metadata)?;
    let mut archive = archive.clone();
    apply_capture_preview_with_retention(&mut archive, preview, None, policy)
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
    let mut archive = archive.clone();
    apply_capture_preview_with_retention(&mut archive, preview, Some(tag), policy)
}

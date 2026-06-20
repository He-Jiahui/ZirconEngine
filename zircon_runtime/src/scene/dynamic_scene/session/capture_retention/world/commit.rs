use crate::scene::World;

use super::super::super::{
    slot_capture, RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata,
};
use super::super::apply::apply_capture_preview_with_retention;

pub(in crate::scene::dynamic_scene::session) fn capture_world_slot_with_retention(
    archive: &mut RuntimeSessionArchive,
    slot_id: impl Into<String>,
    world: &World,
    metadata: RuntimeSessionMetadata,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
    let preview = slot_capture::preview_world_slot(archive, slot_id, world, metadata)?;
    apply_capture_preview_with_retention(archive, preview, None, policy)
}

pub(in crate::scene::dynamic_scene::session) fn capture_world_slot_with_tag_retention(
    archive: &mut RuntimeSessionArchive,
    tag: &str,
    slot_id: impl Into<String>,
    world: &World,
    metadata: RuntimeSessionMetadata,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
    let preview = slot_capture::preview_world_slot(archive, slot_id, world, metadata)?;
    apply_capture_preview_with_retention(archive, preview, Some(tag), policy)
}

use std::path::Path;

use crate::scene::World;

use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata, io,
};

impl RuntimeSessionArchive {
    pub fn capture_world_slot_with_tag_retention_to_path_atomically(
        path: impl AsRef<Path>,
        tag: &str,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        let report =
            archive.capture_world_slot_with_tag_retention(tag, slot_id, world, metadata, policy)?;
        io::save_to_path_atomically(&archive, path)?;
        Ok(report)
    }
}

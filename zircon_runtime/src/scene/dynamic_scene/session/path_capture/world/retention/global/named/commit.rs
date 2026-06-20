use std::path::Path;

use crate::scene::World;

use super::super::super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata,
};

impl RuntimeSessionArchive {
    pub fn capture_world_slot_with_retention_to_path_atomically(
        path: impl AsRef<Path>,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        let report = archive.capture_world_slot_with_retention(slot_id, world, metadata, policy)?;
        io::save_to_path_atomically(&archive, path)?;
        Ok(report)
    }
}

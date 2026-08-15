use std::path::Path;

use crate::scene::World;

use super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionMetadata,
};

impl RuntimeSessionArchive {
    pub fn capture_world_slot_to_path_atomically(
        path: impl AsRef<Path>,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        archive.capture_world_slot(slot_id, world, metadata)?;
        io::save_to_path_atomically(&archive, path)?;
        archive.manifest()
    }
}

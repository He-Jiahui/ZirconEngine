use std::path::Path;

use crate::scene::LevelSystem;

use super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
};

impl RuntimeSessionArchive {
    pub fn capture_level_slot_to_path_atomically(
        path: impl AsRef<Path>,
        slot_id: impl Into<String>,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        archive.capture_level_slot(slot_id, level)?;
        io::save_to_path_atomically(&archive, path)?;
        archive.manifest()
    }
}

use std::path::Path;

use crate::scene::LevelSystem;

use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionSlotSelector, io,
};

impl RuntimeSessionArchive {
    pub fn capture_level_selected_slot_to_path_atomically(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        archive.capture_level_selected_slot(selector, level)?;
        io::save_to_path_atomically(&archive, path)?;
        archive.manifest()
    }

    pub fn capture_level_selected_slot_preserving_metadata_to_path_atomically(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        archive.capture_level_selected_slot_preserving_metadata(selector, level)?;
        io::save_to_path_atomically(&archive, path)?;
        archive.manifest()
    }
}

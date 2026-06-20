use std::fs;
use std::path::Path;

use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::support::ensure_parent_dir;
use super::preview;

pub(in crate::scene::dynamic_scene::session) fn save_to_path(
    archive: &RuntimeSessionArchive,
    path: impl AsRef<Path>,
) -> Result<(), RuntimeSessionArchiveError> {
    let path = path.as_ref();
    preview::preview_save_to_path(archive, path)?;
    ensure_parent_dir(path)?;
    fs::write(path, archive.to_versioned_json_pretty()?)?;
    Ok(())
}

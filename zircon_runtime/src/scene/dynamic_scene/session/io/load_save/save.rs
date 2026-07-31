use std::path::Path;

use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::atomic;

pub(in crate::scene::dynamic_scene::session) fn save_to_path(
    archive: &RuntimeSessionArchive,
    path: impl AsRef<Path>,
) -> Result<(), RuntimeSessionArchiveError> {
    atomic::save_to_path_atomically(archive, path)
}

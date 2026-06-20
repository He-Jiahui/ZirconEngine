use std::path::Path;

use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
};
use super::{load_from_path, save_to_path_atomically};

pub(in crate::scene::dynamic_scene::session) fn mutate_archive_at_path_atomically(
    path: impl AsRef<Path>,
    mutate: impl FnOnce(&mut RuntimeSessionArchive) -> Result<(), RuntimeSessionArchiveError>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    mutate_archive_at_path_with_report_atomically(path, |archive| {
        mutate(archive)?;
        archive.manifest()
    })
}

pub(in crate::scene::dynamic_scene::session) fn mutate_archive_at_path_with_report_atomically<T>(
    path: impl AsRef<Path>,
    mutate: impl FnOnce(&mut RuntimeSessionArchive) -> Result<T, RuntimeSessionArchiveError>,
) -> Result<T, RuntimeSessionArchiveError> {
    let path = path.as_ref();
    let mut archive = load_from_path(path)?;
    let report = mutate(&mut archive)?;
    save_to_path_atomically(&archive, path)?;
    Ok(report)
}

use std::fs;
use std::io;
use std::path::Path;

use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};

pub(in crate::scene::dynamic_scene::session) fn load_from_path(
    path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    RuntimeSessionArchive::from_versioned_json(&fs::read_to_string(path)?)
}

pub(in crate::scene::dynamic_scene::session) fn load_or_empty_from_path(
    path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    match fs::read_to_string(path) {
        Ok(json) => RuntimeSessionArchive::from_versioned_json(&json),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(RuntimeSessionArchive::empty()),
        Err(error) => Err(error.into()),
    }
}

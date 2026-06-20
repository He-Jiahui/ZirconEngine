use std::fs;
use std::io as std_io;
use std::path::Path;

use super::super::{RuntimeSessionArchive, RuntimeSessionArchivePathStatus};

pub(in crate::scene::dynamic_scene::session) fn inspect_path(
    path: impl AsRef<Path>,
) -> RuntimeSessionArchivePathStatus {
    match fs::read_to_string(path) {
        Ok(json) => match RuntimeSessionArchive::from_versioned_json(&json)
            .and_then(|archive| archive.manifest())
        {
            Ok(manifest) => RuntimeSessionArchivePathStatus::Available { manifest },
            Err(error) => RuntimeSessionArchivePathStatus::Invalid { error },
        },
        Err(error) if error.kind() == std_io::ErrorKind::NotFound => {
            RuntimeSessionArchivePathStatus::Missing
        }
        Err(error) => RuntimeSessionArchivePathStatus::Invalid {
            error: error.into(),
        },
    }
}

use std::path::Path;

use super::super::{io, RuntimeSessionArchiveError, RuntimeSessionArchivePathStatus};

pub(in crate::scene::dynamic_scene::session) fn inspect_path(
    path: impl AsRef<Path>,
) -> RuntimeSessionArchivePathStatus {
    let path = path.as_ref();
    match io::load_from_path(path).and_then(|archive| archive.manifest()) {
        Ok(manifest) => RuntimeSessionArchivePathStatus::Available { manifest },
        Err(RuntimeSessionArchiveError::Io(error))
            if error.kind() == std::io::ErrorKind::NotFound =>
        {
            RuntimeSessionArchivePathStatus::Missing
        }
        Err(error) => RuntimeSessionArchivePathStatus::Invalid { error },
    }
}

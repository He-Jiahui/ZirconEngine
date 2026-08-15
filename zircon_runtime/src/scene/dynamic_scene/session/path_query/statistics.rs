use std::path::Path;

use super::super::{io as archive_io, RuntimeSessionArchiveError, RuntimeSessionArchiveStatistics};

pub(in crate::scene::dynamic_scene::session) fn statistics_from_path(
    path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchiveStatistics, RuntimeSessionArchiveError> {
    archive_io::load_from_path(path)?.statistics()
}

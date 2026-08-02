use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveStatistics,
};

pub(in crate::scene::dynamic_scene::session) fn statistics(
    archive: &RuntimeSessionArchive,
) -> Result<RuntimeSessionArchiveStatistics, RuntimeSessionArchiveError> {
    Ok(archive.sealed_artifact()?.statistics().clone())
}

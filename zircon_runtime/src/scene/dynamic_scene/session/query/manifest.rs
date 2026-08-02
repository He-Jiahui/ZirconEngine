use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
};

pub(in crate::scene::dynamic_scene::session) fn manifest(
    archive: &RuntimeSessionArchive,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    Ok(archive.sealed_artifact()?.manifest().clone())
}

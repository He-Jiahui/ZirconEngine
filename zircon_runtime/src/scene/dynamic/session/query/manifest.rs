use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
};

// 清单必须从已封存的代际生成，才能与随后写入的规范字节共享同一验证结果。
pub(in crate::scene::dynamic_scene::session) fn manifest(
    archive: &RuntimeSessionArchive,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    Ok(archive.sealed_artifact()?.manifest().clone())
}

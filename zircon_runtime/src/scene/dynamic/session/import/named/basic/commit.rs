use std::path::Path;

use super::super::super::super::super::super::{
    io, target_path as archive_target_path, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest,
};
use super::super::super::super::loaded::import_slot_from_archive_at_path_atomically;

pub(in crate::scene::dynamic_scene::session) fn import_slot_from_archive_path_at_path_atomically(
    path: impl AsRef<Path>,
    source_path: impl AsRef<Path>,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    // 先比较路径值及已有路径的规范化结果，再加载当次来源快照；检查不锁定随后路径解析。
    // 目标读改写复用 loaded 入口，来源只读；路径比较和来源加载失败时尚未进入目标提交。
    let path = path.as_ref();
    let source_path = source_path.as_ref();
    archive_target_path::reject_same_archive_paths(
        source_path,
        path,
        "runtime session single-slot archive import",
    )?;
    let incoming = io::load_from_path(source_path)?;
    import_slot_from_archive_at_path_atomically(path, &incoming, source_slot_id, new_slot_id)
}

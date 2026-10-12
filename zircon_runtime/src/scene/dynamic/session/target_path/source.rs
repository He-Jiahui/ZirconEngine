use std::fs;
use std::io;
use std::path::Path;

use super::super::RuntimeSessionArchiveError;

// 路径到路径的传输先拒绝同一档案，避免把来源同时当成写入目标；已有路径别名按规范绝对路径比较。
pub(in crate::scene::dynamic_scene::session) fn reject_same_archive_paths(
    source_path: &Path,
    target_path: &Path,
    operation_label: &str,
) -> Result<(), RuntimeSessionArchiveError> {
    if archive_paths_match(source_path, target_path)? {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{operation_label} target archive path must differ from source archive path"),
        )
        .into());
    }
    Ok(())
}

// 不存在的路径无法用文件系统解析别名，只能确认显式相等；其他解析错误需交给调用者处理。
// 这是当次预检观察，不阻止后续文件系统状态或符号链接变化。
fn archive_paths_match(
    source_path: &Path,
    target_path: &Path,
) -> Result<bool, RuntimeSessionArchiveError> {
    if source_path == target_path {
        return Ok(true);
    }

    match (fs::canonicalize(source_path), fs::canonicalize(target_path)) {
        (Ok(source_path), Ok(target_path)) => Ok(source_path == target_path),
        (Err(error), _) if error.kind() != io::ErrorKind::NotFound => Err(error.into()),
        (_, Err(error)) if error.kind() != io::ErrorKind::NotFound => Err(error.into()),
        _ => Ok(false),
    }
}

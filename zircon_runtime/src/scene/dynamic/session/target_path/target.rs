use std::fs;
use std::io;
use std::path::Path;

use super::super::RuntimeSessionArchiveError;

// 为保存与单槽导出预览报告目标是否已有普通文件；缺失父目录由后续保存入口创建。
// 此查询不创建文件，也不预留目标；提交仍需处理状态改变后的文件系统错误。
pub(in crate::scene::dynamic_scene::session) fn target_file_will_replace(
    target_path: &Path,
    target_label: &str,
) -> Result<bool, RuntimeSessionArchiveError> {
    reject_non_directory_parent(target_path, target_label)?;
    match fs::metadata(target_path) {
        Ok(metadata) if metadata.is_file() => Ok(true),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{target_label} path is not a file"),
        )
        .into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

// 在预览阶段排除直接父路径为文件的情况；缺失目录及无显式父路径不视为错误。
fn reject_non_directory_parent(
    target_path: &Path,
    target_label: &str,
) -> Result<(), RuntimeSessionArchiveError> {
    let Some(parent) = target_path.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    match fs::metadata(parent) {
        Ok(metadata) if metadata.is_dir() => Ok(()),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{target_label} parent path is not a directory"),
        )
        .into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

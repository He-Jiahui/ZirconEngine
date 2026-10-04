//! sidecar 名称从源文件名派生并参与导入、删除、重定位的事务；所有写入路径须用同一命名约定定位 .zmeta。

use std::path::{Path, PathBuf};

pub(super) fn meta_path_for_source(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("asset");
    let mut meta_file_name = String::with_capacity(file_name.len() + ".zmeta".len());
    meta_file_name.push_str(file_name);
    meta_file_name.push_str(".zmeta");
    path.with_file_name(meta_file_name)
}

#[cfg(test)]
#[path = "tests/meta_path_for_source.rs"]
mod tests;

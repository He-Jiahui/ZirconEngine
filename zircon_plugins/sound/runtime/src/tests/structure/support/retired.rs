// 构造已退役平铺模块的路径，供结构守卫确认旧模块确实移除；仅计算路径，不读取或删除文件。
use std::path::{Path, PathBuf};

pub(crate) fn retired_flat_module(src: &Path, directory: &str, module: &str) -> PathBuf {
    let file_name = format!("{module}.rs");
    if directory.is_empty() {
        src.join(file_name)
    } else {
        src.join(directory).join(file_name)
    }
}

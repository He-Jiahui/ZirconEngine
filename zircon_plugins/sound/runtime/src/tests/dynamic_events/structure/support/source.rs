// 结构测试读取当前 crate 的生产源码；运行环境须保留编译时清单目录和对应文件，不能只搬运测试二进制。
use std::path::{Path, PathBuf};

pub(crate) fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

pub(super) fn read_source(src: &Path, relative_path: &str) -> String {
    std::fs::read_to_string(src.join(relative_path))
        .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
}

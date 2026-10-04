use std::path::Path;

use crate::projects::project_filesystem_path_key;

/// 为源码检出生成可持久化的绑定 ID；与项目文件系统路径键使用同一规范化规则。
pub fn source_engine_id(source_dir: &Path) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    let key = source_engine_path_key(source_dir);
    for byte in key.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("source-{hash:016x}")
}

/// 登记旧 ID 与新 ID 迁移时比较检出身份，避免同一路径被重复添加。
pub fn same_source_engine_path(left: &Path, right: &Path) -> bool {
    source_engine_path_key(left) == source_engine_path_key(right)
}

/// 首次登记时提供界面名称；已有用户名称由登记调用方保留。
pub fn source_engine_display_name(source_dir: &Path) -> String {
    source_dir
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map(|name| format!("{name} Source"))
        .unwrap_or_else(|| "Local Source".to_string())
}

fn source_engine_path_key(path: &Path) -> String {
    project_filesystem_path_key(path)
}

#[cfg(test)]
#[path = "tests/source_engine_paths.rs"]
mod tests;

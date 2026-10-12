//! 资产入口吸收后，注册、命名空间与查询边界保持由运行时资产模块拥有。向相邻守卫提供源码读取、路径枚举或断言工具。
use std::path::PathBuf;

pub(super) fn runtime_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(super) fn workspace_root() -> PathBuf {
    runtime_root()
        .parent()
        .expect("zircon_runtime should live under the workspace root")
        .to_path_buf()
}

pub(super) fn read_runtime_file(relative_path: &str) -> String {
    std::fs::read_to_string(runtime_root().join(relative_path)).unwrap_or_default()
}

pub(super) fn read_workspace_file(relative_path: &str) -> String {
    std::fs::read_to_string(workspace_root().join(relative_path)).unwrap_or_default()
}

pub(super) fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    for anchor in required {
        assert!(source.contains(anchor), "{label} should contain `{anchor}`");
    }
}

pub(super) fn assert_not_contains_all(label: &str, source: &str, forbidden: &[&str]) {
    for anchor in forbidden {
        assert!(
            !source.contains(anchor),
            "{label} should not contain `{anchor}`"
        );
    }
}

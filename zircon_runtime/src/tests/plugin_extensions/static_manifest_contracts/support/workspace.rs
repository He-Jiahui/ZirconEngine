//! 静态清单回归统一从运行时清单目录的仓库父目录定位插件工作区，避免测试工作目录改变读取来源。
use std::path::{Path, PathBuf};

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn plugins_workspace_root(
) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime crate should have a repository parent")
        .join("zircon_plugins")
}

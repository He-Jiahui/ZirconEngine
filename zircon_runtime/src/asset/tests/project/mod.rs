//! 项目资产测试共享临时项目根，并按资产图、持久缓存、管理器、模板 URI 与 sidecar 生命周期分组。

#[cfg(feature = "graphics")]
mod asset_flow_sample;
mod binary_artifact_cache;
mod binary_artifact_cache_assertions;
mod example_vampire;
mod manager;
mod manifest;
mod package_assets;
mod template_contract;
mod uri;
mod zmeta;

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// 所有项目夹具写入 Cargo target 下的测试输出区；调用方负责创建布局并在结束时清理。
pub(crate) fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_root = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"));
    output_root
        .join("zircon-test-output")
        .join(format!("zircon_asset_{label}_{unique}"))
}

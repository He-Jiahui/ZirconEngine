//! 项目资产测试共享临时项目根，并按资产图、持久缓存、管理器、模板 URI 与 sidecar 生命周期分组。

#[cfg(feature = "graphics")]
#[path = "asset_flow_sample.rs"]
mod asset_flow_sample;
#[path = "binary_artifact_cache.rs"]
mod binary_artifact_cache;
#[path = "binary_artifact_cache_assertions.rs"]
mod binary_artifact_cache_assertions;
#[path = "example_vampire.rs"]
mod example_vampire;
#[path = "manager.rs"]
mod manager;
#[path = "manifest.rs"]
mod manifest;
#[path = "package_assets.rs"]
mod package_assets;
#[path = "template_contract.rs"]
mod template_contract;
#[path = "uri.rs"]
mod uri;
#[path = "zmeta.rs"]
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

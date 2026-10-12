//! 将编译结果和验证报告交付为包数据及其清单；宿主策略报告与运行时加载/执行阶段保留各自职责。
#[path = "artifact.rs"]
mod artifact;
#[path = "header.rs"]
mod header;
#[path = "manifest.rs"]
mod manifest;
#[path = "package_manifest.rs"]
mod package_manifest;
#[path = "report.rs"]
mod report;
#[path = "validate.rs"]
mod validate;

pub use artifact::UiRuntimeCompiledAssetArtifact;
pub use package_manifest::compiled_asset_package_manifest_from_artifact_bytes;

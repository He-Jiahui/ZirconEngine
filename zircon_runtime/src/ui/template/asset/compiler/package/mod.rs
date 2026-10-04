//! 将编译结果和验证报告交付为包数据及其清单；宿主策略报告与运行时加载/执行阶段保留各自职责。
mod artifact;
mod header;
mod manifest;
mod package_manifest;
mod report;
mod validate;

pub use artifact::UiRuntimeCompiledAssetArtifact;
pub use package_manifest::compiled_asset_package_manifest_from_artifact_bytes;

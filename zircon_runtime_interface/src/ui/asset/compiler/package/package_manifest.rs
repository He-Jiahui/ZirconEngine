use serde::{Deserialize, Serialize};

use crate::ui::template::{
    UiAssetError, UiAssetFingerprint, UiCompiledAssetArtifact, UiCompiledAssetCacheRecord,
    UiCompiledAssetDependencyManifest, UiCompiledAssetHeader,
    UI_COMPILED_ASSET_TOML_ENVELOPE_SCHEMA_VERSION,
};

/// 描述编译产物的 TOML 清单，关联报告身份、依赖、缓存键与产物字节摘要。
/// 清单恢复和产物校验是两步操作；`import_toml` 只反序列化此元数据。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiCompiledAssetPackageManifest {
    pub header: UiCompiledAssetHeader,
    pub dependencies: UiCompiledAssetDependencyManifest,
    pub cache: UiCompiledAssetCacheRecord,
    pub artifact: UiCompiledAssetPackageArtifactEntry,
}

impl UiCompiledAssetPackageManifest {
    // TODO: [CR-UITEMPLATE-0002] 运行时使用自己的具体产物和同形转换函数；
    // 确认保留两套清单构造入口的意图，防止后续版本规则分叉。
    /// 按接口层产物和最终字节生成清单，调用者须保证两个参数对应。
    pub fn from_artifact_bytes(artifact: &UiCompiledAssetArtifact, artifact_bytes: &[u8]) -> Self {
        let artifact_fingerprint = UiAssetFingerprint::from_bytes(artifact_bytes);
        Self {
            header: artifact.report.header.clone(),
            dependencies: artifact.report.dependencies.clone(),
            cache: UiCompiledAssetCacheRecord::from_artifact_bytes(artifact, artifact_bytes),
            artifact: UiCompiledAssetPackageArtifactEntry {
                schema_version: UI_COMPILED_ASSET_TOML_ENVELOPE_SCHEMA_VERSION,
                byte_len: artifact_bytes.len() as u64,
                fingerprint: artifact_fingerprint,
            },
        }
    }

    pub fn write_toml(&self) -> Result<String, UiAssetError> {
        toml::to_string(self).map_err(package_manifest_error)
    }

    /// 仅解析 TOML；不会打开产物或核对 schema、长度、指纹和缓存键。
    pub fn import_toml(source: &str) -> Result<Self, UiAssetError> {
        toml::from_str(source).map_err(package_manifest_error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiCompiledAssetPackageArtifactEntry {
    pub schema_version: u32,
    pub byte_len: u64,
    pub fingerprint: UiAssetFingerprint,
}

fn package_manifest_error(error: impl std::fmt::Display) -> UiAssetError {
    UiAssetError::InvalidDocument {
        asset_id: "ui-compiled-package-manifest".to_string(),
        detail: error.to_string(),
    }
}

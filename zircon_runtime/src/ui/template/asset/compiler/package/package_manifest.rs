use serde::{Deserialize, Serialize};

use crate::ui::template::{UiCompiledAssetArtifact, UiCompiledAssetCacheRecord};
use zircon_runtime_interface::ui::template::{
    UiAssetError, UiAssetFingerprint, UiCompiledAssetDependencyManifest, UiCompiledAssetHeader,
    UI_COMPILED_ASSET_BINARY_ARTIFACT_SCHEMA_VERSION,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiCompiledAssetPackageManifest {
    pub header: UiCompiledAssetHeader,
    pub dependencies: UiCompiledAssetDependencyManifest,
    pub cache: UiCompiledAssetCacheRecord,
    pub artifact: UiCompiledAssetPackageArtifactEntry,
}

impl UiCompiledAssetPackageManifest {
    pub fn from_artifact_bytes(artifact: &UiCompiledAssetArtifact, artifact_bytes: &[u8]) -> Self {
        let artifact_fingerprint = UiAssetFingerprint::from_bytes(artifact_bytes);
        Self {
            header: artifact.report.header.clone(),
            dependencies: artifact.report.dependencies.clone(),
            cache: UiCompiledAssetCacheRecord::from_artifact_bytes(artifact, artifact_bytes),
            artifact: UiCompiledAssetPackageArtifactEntry {
                schema_version: UI_COMPILED_ASSET_BINARY_ARTIFACT_SCHEMA_VERSION,
                byte_len: artifact_bytes.len() as u64,
                fingerprint: artifact_fingerprint,
            },
        }
    }

    pub fn write_toml(&self) -> Result<String, UiAssetError> {
        toml::to_string(self).map_err(package_manifest_error)
    }

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

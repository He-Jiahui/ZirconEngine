use crate::ui::template::UiRuntimeCompiledAssetArtifact;
use zircon_runtime_interface::ui::template::{
    UiAssetFingerprint, UiCompiledAssetCacheRecord, UiCompiledAssetPackageArtifactEntry,
    UiCompiledAssetPackageManifest, UI_COMPILED_ASSET_BINARY_ARTIFACT_SCHEMA_VERSION,
};

pub fn compiled_asset_package_manifest_from_artifact_bytes(
    artifact: &UiRuntimeCompiledAssetArtifact,
    artifact_bytes: &[u8],
) -> UiCompiledAssetPackageManifest {
    let artifact_fingerprint = UiAssetFingerprint::from_bytes(artifact_bytes);
    UiCompiledAssetPackageManifest {
        header: artifact.report.header.clone(),
        dependencies: artifact.report.dependencies.clone(),
        cache: compiled_asset_cache_record_from_artifact_bytes(artifact, artifact_bytes),
        artifact: UiCompiledAssetPackageArtifactEntry {
            schema_version: UI_COMPILED_ASSET_BINARY_ARTIFACT_SCHEMA_VERSION,
            byte_len: artifact_bytes.len() as u64,
            fingerprint: artifact_fingerprint,
        },
    }
}

fn compiled_asset_cache_record_from_artifact_bytes(
    artifact: &UiRuntimeCompiledAssetArtifact,
    artifact_bytes: &[u8],
) -> UiCompiledAssetCacheRecord {
    let cache_key = artifact.report.header.compile_cache_key.clone();
    UiCompiledAssetCacheRecord {
        header: artifact.report.header.clone(),
        invalidation_snapshot: cache_key.invalidation_snapshot(),
        cache_key,
        artifact_fingerprint: UiAssetFingerprint::from_bytes(artifact_bytes),
        artifact_byte_len: artifact_bytes.len() as u64,
    }
}

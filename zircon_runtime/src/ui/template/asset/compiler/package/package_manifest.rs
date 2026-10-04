use crate::ui::template::UiRuntimeCompiledAssetArtifact;
use zircon_runtime_interface::ui::template::{
    UiAssetFingerprint, UiCompiledAssetCacheRecord, UiCompiledAssetPackageArtifactEntry,
    UiCompiledAssetPackageManifest, UI_COMPILED_ASSET_TOML_ENVELOPE_SCHEMA_VERSION,
};

/// 为调用方实际写出的包字节生成完整清单；artifact 与字节必须来自同一次序列化，此入口不会反解字节核对二者。
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
            schema_version: UI_COMPILED_ASSET_TOML_ENVELOPE_SCHEMA_VERSION,
            byte_len: artifact_bytes.len() as u64,
            fingerprint: artifact_fingerprint,
        },
    }
}

// 复用输入键描述何时失效，产物指纹描述交付的具体字节；二者服务于不同检查，不能互换。
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

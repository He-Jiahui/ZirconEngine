use serde::{Deserialize, Serialize};

use crate::ui::template::{
    UiAssetFingerprint, UiCompileCacheKey, UiCompiledAssetArtifact, UiCompiledAssetHeader,
    UiInvalidationSnapshot,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiCompiledAssetCacheRecord {
    pub header: UiCompiledAssetHeader,
    pub cache_key: UiCompileCacheKey,
    pub invalidation_snapshot: UiInvalidationSnapshot,
    pub artifact_fingerprint: UiAssetFingerprint,
    pub artifact_byte_len: u64,
}

impl UiCompiledAssetCacheRecord {
    pub fn from_artifact_bytes(artifact: &UiCompiledAssetArtifact, artifact_bytes: &[u8]) -> Self {
        let cache_key = artifact.report.header.compile_cache_key.clone();
        Self {
            header: artifact.report.header.clone(),
            invalidation_snapshot: cache_key.invalidation_snapshot(),
            cache_key,
            artifact_fingerprint: UiAssetFingerprint::from_bytes(artifact_bytes),
            artifact_byte_len: artifact_bytes.len() as u64,
        }
    }
}

use serde::{Deserialize, Serialize};

use crate::ui::template::{
    UiAssetFingerprint, UiCompileCacheKey, UiCompiledAssetArtifact, UiCompiledAssetHeader,
    UiInvalidationSnapshot,
};

/// 包清单中的缓存元数据，关联编译键、失效快照和序列化字节的指纹与长度。
/// 此类型可由接口层产物构造；运行时包路径也会填入同形记录。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiCompiledAssetCacheRecord {
    pub header: UiCompiledAssetHeader,
    pub cache_key: UiCompileCacheKey,
    pub invalidation_snapshot: UiInvalidationSnapshot,
    pub artifact_fingerprint: UiAssetFingerprint,
    pub artifact_byte_len: u64,
}

impl UiCompiledAssetCacheRecord {
    /// 传入的字节必须是该产物最终写出的内容；本函数不会核对二者是否对应。
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

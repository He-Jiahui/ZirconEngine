//! 将稳定资源 ID 与修订号配对，供剪辑评估缓存和诊断去重识别一次具体资产版本。
use zircon_runtime::core::resource::ResourceId;

/// Stable asset identity paired with the resource revision used for cache invalidation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// 缓存键由调用方从实际快照取得；手工保持旧修订会把新载荷误认为缓存命中。
pub struct AnimationAssetRevision {
    id: ResourceId,
    revision: u64,
}

impl AnimationAssetRevision {
    pub const fn new(id: ResourceId, revision: u64) -> Self {
        Self { id, revision }
    }

    pub const fn id(self) -> ResourceId {
        self.id
    }

    pub const fn revision(self) -> u64 {
        self.revision
    }
}

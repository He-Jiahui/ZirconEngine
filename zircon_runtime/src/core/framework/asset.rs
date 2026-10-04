use std::sync::Arc;

use crate::core::resource::{
    ResourceEventReceiver, ResourceManagementGeneration, ResourceRecord, ResourceState,
};

/// Minimal invalidation stamp for caches that do not need a cloned resource record.
///
/// State is part of the identity because reload recovery may reach `Ready` without changing the
/// content revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceCacheIdentity {
    pub revision: u64,
    pub state: ResourceState,
}

/// 给 Editor 的资源句柄解析和 Runtime 的字体缓存提供项目资源只读视图。
/// 调用方以资源 locator 查询，具体项目管理器负责解析；返回 None 同时覆盖无效 locator 与未登记资源。
/// 订阅变化后仍需重新读取状态或世代，不能把通知当作完整资源快照。
pub trait ResourceManager: Send + Sync {
    fn resolve_resource_id(&self, locator: &str) -> Option<String>;
    fn resource_status(&self, locator: &str) -> Option<ResourceRecord>;
    fn resource_management_generation(&self) -> Arc<ResourceManagementGeneration>;
    fn resource_revision(&self, locator: &str) -> Option<u64>;
    /// 缓存只关心版本与就绪状态时使用；实现者可覆盖默认的完整记录克隆路径。
    fn resource_cache_identity(&self, locator: &str) -> Option<ResourceCacheIdentity> {
        self.resource_status(locator)
            .map(|record| ResourceCacheIdentity {
                revision: record.revision,
                state: record.state,
            })
    }
    fn subscribe_resource_changes(&self) -> ResourceEventReceiver;
}

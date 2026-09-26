use serde::{Deserialize, Serialize};

use super::{ResourceEventKind, ResourceId, ResourceKind, ResourceLocator};

/// 资源事务提交后发布的身份与修订通知，供 Runtime 和 Editor 增量刷新。
/// 更名时 `previous_locator` 指向旧索引键；接收端仍需处理事件流缺口。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceEvent {
    pub kind: ResourceEventKind,
    pub resource_kind: ResourceKind,
    pub id: ResourceId,
    pub locator: Option<ResourceLocator>,
    pub previous_locator: Option<ResourceLocator>,
    pub revision: u64,
}

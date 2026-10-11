use serde::{Deserialize, Serialize};

use super::{ResourceEventKind, ResourceId, ResourceKind, ResourceLocator};

/// 资源事务提交后发布的身份与修订通知，供 Runtime 和 Editor 增量刷新。
/// 更名时 `previous_locator` 指向旧索引键；接收端仍需处理事件流缺口。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 携带资源提交前后目录记录的净变化，供界面和资源视图决定刷新哪些身份或地址。
///
/// # 字段与时间边界
///
/// `kind` 说明变化，`resource_kind` 与 `id` 指向受影响资源；`locator` 为所投影记录的地址，重命名可通过 `previous_locator` 保留旧地址。移除事件使用被移除的记录。
///
/// # 消费职责
///
/// `revision` 是资源修订，日志顺序由独立序号表示。有界日志可合并或淘汰事件；缺口后的完整状态应由读取端重新取得，单个事件不能证明当前状态仍未变化。
///
/// # 参考
///
/// [`crate::resource::ResourceEventKind`]、[`crate::resource::ResourceRecord`]、[`crate::resource::ResourceId`]。
pub struct ResourceEvent {
    pub kind: ResourceEventKind,
    pub resource_kind: ResourceKind,
    pub id: ResourceId,
    pub locator: Option<ResourceLocator>,
    pub previous_locator: Option<ResourceLocator>,
    /// 此资源的修订；移除事件沿用被移除记录的修订，日志顺序由独立序列号表示。
    pub revision: u64,
}

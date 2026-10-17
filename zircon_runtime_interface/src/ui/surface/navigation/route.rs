use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::UiNavigationEventKind;

/// Runtime 优先以焦点节点、其次以导航根构造路由；分发器按 bubbled 顺序查找处理器。
/// 两者都不存在时，root_targets 是根节点回退候选，fallback_to_root 记录这一选择。
///
/// 冒泡和根目标数组由路由拥有，标识不会延长节点生命期；反序列化不验证目标与数组的一致性。
/// 上面的回退规则描述 Surface 的构造路径，手动构造该值不会触发回退派发。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNavigationRoute {
    pub kind: UiNavigationEventKind,
    pub target: Option<UiNodeId>,
    pub bubbled: Vec<UiNodeId>,
    pub fallback_to_root: bool,
    pub root_targets: Vec<UiNodeId>,
}

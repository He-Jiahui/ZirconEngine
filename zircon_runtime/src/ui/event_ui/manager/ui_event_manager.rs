use std::collections::{BTreeMap, HashMap};

use crossbeam_channel::Sender;

use super::route_entry::RouteEntry;
use zircon_runtime_interface::ui::event_ui::{
    UiNodeId, UiNodePath, UiNotification, UiReflectionSnapshot, UiRouteId, UiSubscriptionId,
    UiTreeId,
};

/// 拥有 UI 路由注册表、反射快照、节点路径索引和通知发送端的同步控制面状态。
///
/// # 使用时机
/// 宿主注册处理器或宿主动作绑定，再发布快照供查询、动作定位和反射更新使用。
/// 仅登记宿主动作的路由没有执行回调，宿主须在自己的请求分派层执行该动作。
///
/// # 约束与生命周期
/// 管理器持有注册回调及其捕获，以及拥有的反射快照；查询结果由调用者独立拥有。
/// 原生绑定键指向最近注册的路由，历史路由编号仍保留在编号表中。
/// 通知接收端由订阅者持有；保存管理器或订阅编号不会代替消费接收端。
///
/// 参见 [`Self::register_route`]、[`Self::replace_tree`] 和 [`Self::subscribe`]。
#[derive(Default)]
pub struct UiEventManager {
    pub(super) next_route_id: u64,
    pub(super) next_subscription_id: u64,
    pub(super) routes_by_id: BTreeMap<UiRouteId, RouteEntry>,
    pub(super) routes_by_binding: BTreeMap<String, UiRouteId>,
    pub(super) trees: BTreeMap<UiTreeId, UiReflectionSnapshot>,
    pub(super) node_index: HashMap<UiNodePath, (UiTreeId, UiNodeId)>,
    pub(super) subscriptions: BTreeMap<UiSubscriptionId, Sender<UiNotification>>,
}

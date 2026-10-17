use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::UiNavigationRoute;

/// Ephemeral handler view over the route that becomes the dispatch result authority.
/// Runtime 为当前目标或冒泡祖先创建此视图；处理器借用路由判断意图，再返回派发决议。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiNavigationDispatchContext<'route> {
    /// 当前被调用处理器所属节点。
    pub node_id: UiNodeId,
    /// 借用的整条导航路由；引用仅在本次处理器调用期间有效。
    pub route: &'route UiNavigationRoute,
}

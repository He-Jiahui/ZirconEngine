use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::UiNavigationRoute;

/// Ephemeral handler view over the route that becomes the dispatch result authority.
/// Runtime 为当前目标或冒泡祖先创建此视图；处理器借用路由判断意图，再返回派发决议。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiNavigationDispatchContext<'route> {
    pub node_id: UiNodeId,
    pub route: &'route UiNavigationRoute,
}

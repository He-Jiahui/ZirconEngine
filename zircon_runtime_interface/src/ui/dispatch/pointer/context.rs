use crate::ui::dispatch::UiDispatchPhase;
use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::UiPointerRoute;

/// Ephemeral handler view over the route that becomes the dispatch result authority.
/// Runtime 按预览、目标或冒泡阶段借出同一路由；处理器据此选择效果，状态由 UiSurface 提交。
///
/// # 生命周期
/// Runtime 回调中的 `route` 借用来自本次派发；注册闭包的 `'static` 约束不会延长它。
/// 返回的效果不携带该借用；若需跨回调保存路由信息，应保存自己拥有的数据。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiPointerDispatchContext<'route> {
    pub node_id: UiNodeId,
    pub phase: UiDispatchPhase,
    pub route: &'route UiPointerRoute,
}

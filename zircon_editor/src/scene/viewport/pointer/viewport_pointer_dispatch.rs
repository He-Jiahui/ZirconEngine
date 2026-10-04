//! 分发产品携带本次路由和同一次解析的调试报告，宿主用它更新悬停或启动交互，不应独立重算第二份命中。

use zircon_runtime::core::framework::picking::PickingDebugFeed;

use super::viewport_pointer_route::ViewportPointerRoute;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ViewportPointerDispatch {
    pub route: Option<ViewportPointerRoute>,
    pub picking_debug_feed: Option<PickingDebugFeed>,
}

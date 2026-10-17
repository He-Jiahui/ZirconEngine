use super::route_handler::RouteHandler;
use zircon_runtime_interface::ui::binding::UiEventBinding;
use zircon_runtime_interface::ui::event_ui::UiRouteId;

// handler 为 None 表示路由由宿主 typed dispatcher 执行；并非一个可由本管理器直接执行的空处理器。
/// 路由表拥有的注册记录；绑定描述与编号随记录保存。
/// 克隆记录会克隆绑定值并共享处理器的 Arc，不会执行回调。
#[derive(Clone)]
pub(super) struct RouteEntry {
    pub(super) route_id: UiRouteId,
    pub(super) binding: UiEventBinding,
    pub(super) handler: Option<RouteHandler>,
}

use super::route_handler::RouteHandler;
use zircon_runtime_interface::ui::binding::UiEventBinding;
use zircon_runtime_interface::ui::event_ui::UiRouteId;

#[derive(Clone)]
pub(super) struct RouteEntry {
    pub(super) route_id: UiRouteId,
    pub(super) binding: UiEventBinding,
    pub(super) handler: Option<RouteHandler>,
}

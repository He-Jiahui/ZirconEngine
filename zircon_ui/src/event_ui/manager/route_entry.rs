use crate::UiEventBinding;

use super::super::UiRouteId;
use super::route_handler::RouteHandler;

#[derive(Clone)]
pub(super) struct RouteEntry {
    pub(super) route_id: UiRouteId,
    pub(super) binding: UiEventBinding,
    pub(super) handler: Option<RouteHandler>,
}

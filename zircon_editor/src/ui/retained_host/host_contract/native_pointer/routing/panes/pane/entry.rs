mod template;
mod viewport_body;

use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};

use self::template::route_template_node_hit;
use self::viewport_body::viewport_body_route;
use super::super::super::{geometry::contains, PanePointerRoute, PanePointerTarget};
use super::super::mode::PaneRouteMode;
use super::super::target::pane_pointer_target_for_kind;

pub(in super::super) fn pane_route_from_pane(
    pane: &PaneData,
    content: &FrameRect,
    x: f32,
    y: f32,
    surface_key: Option<&str>,
    mode: PaneRouteMode,
) -> Option<PanePointerRoute> {
    if !contains(content, x, y) {
        return None;
    }
    let body_route = viewport_body_route(pane, content, x, y, surface_key);
    if let Some(toolbar_route) = body_route.toolbar_route {
        return Some(toolbar_route);
    }
    if let Some(route) = route_template_node_hit(pane, &body_route.body, x, y, mode) {
        return Some(route);
    }
    let target = pane_pointer_target_for_kind(pane, surface_key);
    Some(PanePointerRoute::new(target, &body_route.body, x, y))
}

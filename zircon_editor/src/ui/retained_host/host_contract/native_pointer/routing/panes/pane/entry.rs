mod asset_content;
mod asset_reference;
mod asset_tree;
mod console;
mod template;
mod viewport_body;

use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};

use self::asset_content::route_asset_content_hit;
use self::asset_reference::route_asset_reference_hit;
use self::asset_tree::route_browser_asset_tree_hit;
use self::console::console_output_route_frame;
use self::template::route_template_node_hit;
use self::viewport_body::viewport_body_route;
use super::super::super::{PanePointerRoute, PanePointerTarget, geometry::contains};
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
    if let Some(route) = route_browser_asset_tree_hit(pane, &body_route.body, x, y) {
        return Some(route);
    }
    if let Some(route) = route_asset_reference_hit(pane, &body_route.body, x, y) {
        return Some(route);
    }
    if let Some(route) = route_asset_content_hit(pane, &body_route.body, x, y) {
        return Some(route);
    }
    if let Some(route) = route_template_node_hit(pane, &body_route.body, x, y, mode) {
        return Some(route);
    }
    let target = pane_pointer_target_for_kind(pane, surface_key);
    if mode.uses_console_output_viewport() && matches!(&target, PanePointerTarget::Console) {
        let route_frame = console_output_route_frame(pane, &body_route.body)
            .unwrap_or_else(|| body_route.body.clone());
        return contains(&route_frame, x, y)
            .then(|| PanePointerRoute::new(target, &route_frame, x, y));
    }
    Some(PanePointerRoute::new(target, &body_route.body, x, y))
}

#[cfg(test)]
#[path = "entry/tests.rs"]
mod tests;

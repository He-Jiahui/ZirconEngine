#[path = "../../../../../../contract/pointer/routing/panes/pane/entry/asset_content.rs"]
mod asset_content;
#[path = "../../../../../../pointer/routing/panes/pane/entry/asset_reference.rs"]
mod asset_reference;
#[path = "../../../../../../contract/pointer/routing/panes/pane/entry/asset_tree.rs"]
mod asset_tree;
#[path = "../../../../../../contract/pointer/routing/panes/pane/entry/console.rs"]
mod console;
#[path = "../../../../../../contract/pointer/routing/panes/pane/entry/template.rs"]
mod template;
#[path = "../../../../../../contract/pointer/routing/panes/pane/entry/viewport_body.rs"]
mod viewport_body;

use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};

use self::asset_content::route_asset_content_hit;
use self::asset_reference::route_asset_reference_hit;
use self::asset_tree::route_browser_asset_tree_hit;
use self::console::console_output_route_frame;
use self::template::route_template_node_hit;
use self::viewport_body::viewport_body_route;
use super::super::super::{geometry::contains, PanePointerRoute, PanePointerTarget};
use super::super::mode::PaneRouteMode;
use super::super::target::pane_pointer_target_for_kind;

pub(in super::super) fn pane_route_from_pane<'a>(
    pane: &'a PaneData,
    content: &FrameRect,
    x: f32,
    y: f32,
    surface_key: Option<&'a str>,
    mode: PaneRouteMode,
    console_scroll_px: f32,
) -> Option<PanePointerRoute<'a>> {
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
    if pane.kind.as_str() == "UiAssetEditor" && pane.ui_asset.preview.canvas.input_enabled {
        let canvas = &pane.ui_asset.preview.canvas;
        if let (Some(source_frame), Some(transform)) = (
            canvas.source_surface_frame.as_ref(),
            canvas.transform(&pane.ui_asset.designer_canvas_panel_node, &body_route.body),
        ) {
            if transform.source_point(x, y).is_some() {
                let canvas_clip = transform.clip.clone();
                return Some(PanePointerRoute::new(
                    PanePointerTarget::UiAssetPreview {
                        pane_id: pane.id.as_str(),
                        source_frame,
                        transform,
                    },
                    &canvas_clip,
                    x,
                    y,
                ));
            }
        }
    }
    if pane.kind.as_str() == "GeneratedBottom" {
        if let (Some(source_frame), Some(transform)) = (
            pane.generated_bottom.source_surface_frame.as_ref(),
            pane.generated_bottom.input_transform(&body_route.body),
        ) {
            if transform.source_point(x, y).is_some() {
                let clip = transform.clip.clone();
                return Some(PanePointerRoute::new(
                    PanePointerTarget::GeneratedBottom {
                        pane_id: pane.id.as_str(),
                        source_frame,
                        transform,
                    },
                    &clip,
                    x,
                    y,
                ));
            }
        }
    }
    if let Some(route) =
        route_template_node_hit(pane, &body_route.body, x, y, mode, console_scroll_px)
    {
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
#[path = "../../../../../../contract/pointer/routing/panes/pane/entry/tests/cases.rs"]
mod tests;

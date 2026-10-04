use super::super::super::{PanePointerRoute, PanePointerTarget};
use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};
use crate::ui::retained_host::host_contract::viewport_chrome_state::{
    control_enabled, mounted_control_id,
};
use zircon_runtime::ui::surface::hit_test_surface_frame;
use zircon_runtime_interface::ui::layout::UiPoint;

pub(super) fn route_viewport_toolbar<'a>(
    pane: &'a PaneData,
    toolbar: &FrameRect,
    x: f32,
    y: f32,
    surface_key: Option<&'a str>,
) -> PanePointerRoute<'a> {
    let surface_key = surface_key.unwrap_or("document");
    let hit_entry = pane
        .viewport
        .toolbar_surface_frame
        .as_ref()
        .and_then(|frame| {
            hit_test_surface_frame(frame, UiPoint::new(x - toolbar.x, y - toolbar.y))
                .top_entry(&frame.hit_grid)
        });
    let Some(entry) = hit_entry else {
        let target = match pane.kind.as_str() {
            "Scene" => PanePointerTarget::SceneViewport(surface_key),
            "Game" => PanePointerTarget::GameViewport(surface_key),
            _ => PanePointerTarget::Other,
        };
        return PanePointerRoute::new(target, toolbar, x, y);
    };
    let source = pane.viewport.toolbar_template_nodes.iter().find(|node| {
        node.frame.x == entry.frame.x
            && node.frame.y == entry.frame.y
            && node.frame.width == entry.frame.width
            && node.frame.height == entry.frame.height
    });
    let enabled = source.is_none_or(|node| {
        !node.disabled && control_enabled(&pane.viewport, node.control_id.as_str())
    });
    let control_frame = FrameRect {
        x: toolbar.x + entry.frame.x,
        y: toolbar.y + entry.frame.y,
        width: entry.frame.width,
        height: entry.frame.height,
    };
    PanePointerRoute::new(
        PanePointerTarget::ViewportToolbar {
            surface_key,
            control_id: enabled.then_some(entry.control_id.as_deref()).flatten(),
            source_control_id: source
                .filter(|_| enabled)
                .map(|node| mounted_control_id(&pane.viewport, node.control_id.as_str())),
            control_frame,
        },
        toolbar,
        x,
        y,
    )
}

#[cfg(test)]
#[path = "tests/toolbar.rs"]
mod tests;

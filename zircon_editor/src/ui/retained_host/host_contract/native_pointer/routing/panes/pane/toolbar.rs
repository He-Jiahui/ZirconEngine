use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};
use zircon_runtime::ui::surface::hit_test_surface_frame;
use zircon_runtime_interface::ui::layout::UiPoint;

use super::super::super::{PanePointerRoute, PanePointerTarget};

pub(super) fn route_viewport_toolbar<'a>(
    pane: &'a PaneData,
    toolbar: &FrameRect,
    x: f32,
    y: f32,
    surface_key: Option<&'a str>,
) -> PanePointerRoute<'a> {
    let surface_key = surface_key.unwrap_or("document");
    let Some(control_id) = viewport_toolbar_control_id(pane, toolbar, x, y) else {
        let target = match pane.kind.as_str() {
            "Scene" => PanePointerTarget::SceneViewport(surface_key),
            "Game" => PanePointerTarget::GameViewport(surface_key),
            _ => PanePointerTarget::Other,
        };
        return PanePointerRoute::new(target, toolbar, x, y);
    };
    PanePointerRoute::new(
        PanePointerTarget::ViewportToolbar {
            surface_key,
            control_id: Some(control_id),
        },
        toolbar,
        x,
        y,
    )
}

fn viewport_toolbar_control_id<'a>(
    pane: &'a PaneData,
    toolbar: &FrameRect,
    x: f32,
    y: f32,
) -> Option<&'a str> {
    let surface_frame = pane.viewport.toolbar_surface_frame.as_ref()?;
    let point = UiPoint::new(x - toolbar.x, y - toolbar.y);
    let hit = hit_test_surface_frame(surface_frame, point);
    hit.top_entry(&surface_frame.hit_grid)?
        .control_id
        .as_deref()
}

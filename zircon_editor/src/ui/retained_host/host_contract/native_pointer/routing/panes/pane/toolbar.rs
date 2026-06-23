use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};
use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime::ui::surface::hit_test_surface_frame;
use zircon_runtime_interface::ui::layout::UiPoint;

use super::super::super::{PanePointerRoute, PanePointerTarget};

pub(super) fn route_viewport_toolbar(
    pane: &PaneData,
    toolbar: &FrameRect,
    x: f32,
    y: f32,
    surface_key: Option<&str>,
) -> PanePointerRoute {
    let surface_key = surface_key.unwrap_or("document");
    let Some(control_id) = viewport_toolbar_control_id(pane, toolbar, x, y) else {
        return PanePointerRoute::new(
            PanePointerTarget::Viewport(surface_key.into()),
            toolbar,
            x,
            y,
        );
    };
    PanePointerRoute::new(
        PanePointerTarget::ViewportToolbar {
            surface_key: surface_key.into(),
            control_id: Some(control_id),
        },
        toolbar,
        x,
        y,
    )
}

fn viewport_toolbar_control_id(
    pane: &PaneData,
    toolbar: &FrameRect,
    x: f32,
    y: f32,
) -> Option<SharedString> {
    let surface_frame = pane.viewport.toolbar_surface_frame.as_ref()?;
    let point = UiPoint::new(x - toolbar.x, y - toolbar.y);
    let node_id = hit_test_surface_frame(surface_frame, point).top_hit?;
    let node = surface_frame.arranged_tree.get(node_id)?;
    node.control_id.clone()
}

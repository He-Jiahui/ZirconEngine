use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};
use crate::ui::retained_host::host_contract::surface_hit_test;

use super::super::super::{PanePointerRoute, PanePointerTarget};

pub(super) fn route_viewport_toolbar(
    pane: &PaneData,
    toolbar: &FrameRect,
    x: f32,
    y: f32,
    surface_key: Option<&str>,
) -> PanePointerRoute {
    let surface_key = surface_key.unwrap_or("document");
    if let Some(hit) =
        surface_hit_test::hit_test_viewport_toolbar(surface_key, &pane.viewport, toolbar, x, y)
    {
        return PanePointerRoute::new(PanePointerTarget::ViewportToolbar(hit), toolbar, x, y);
    }
    PanePointerRoute::new(
        PanePointerTarget::Viewport(surface_key.into()),
        toolbar,
        x,
        y,
    )
}

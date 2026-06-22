use super::super::super::data::{FrameRect, PaneData};
use super::super::super::surface_hit_test;

pub(super) fn viewport_toolbar_route_hit(
    id: &str,
    x: f32,
    y: f32,
    surface_key: &str,
    pane: &PaneData,
    toolbar: &FrameRect,
) -> bool {
    surface_hit_test::hit_test_viewport_toolbar(surface_key, &pane.viewport, toolbar, x, y)
        .is_some_and(|hit| {
            format!("viewport_toolbar_control.{surface_key}.{}", hit.control_id) == id
        })
}

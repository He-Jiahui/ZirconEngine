use crate::ui::retained_host::primitives::SharedString;

use super::super::data::{FrameRect, SceneViewportChromeData};
use super::surface_frame::hit_test_host_surface_frame;

pub(crate) struct ViewportToolbarPointerHit {
    pub(crate) surface_key: SharedString,
    pub(crate) control_id: SharedString,
    pub(crate) control_x: f32,
    pub(crate) control_y: f32,
    pub(crate) control_width: f32,
    pub(crate) control_height: f32,
}

pub(crate) fn hit_test_viewport_toolbar(
    surface_key: &str,
    viewport: &SceneViewportChromeData,
    toolbar: &FrameRect,
    x: f32,
    y: f32,
) -> Option<ViewportToolbarPointerHit> {
    let surface_frame = viewport.toolbar_surface_frame.as_ref()?;
    let hit = hit_test_host_surface_frame(surface_frame, toolbar, x, y)?;
    Some(ViewportToolbarPointerHit {
        surface_key: surface_key.into(),
        control_id: hit.control_id,
        control_x: hit.control_frame.x,
        control_y: hit.control_frame.y,
        control_width: hit.control_frame.width,
        control_height: hit.control_frame.height,
    })
}

#[cfg(test)]
#[path = "viewport_toolbar_tests.rs"]
mod tests;

use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};

use super::super::super::super::super::{geometry::contains, PanePointerRoute};
use super::super::super::toolbar::route_viewport_toolbar;

pub(super) fn viewport_toolbar_frame(pane: &PaneData, content: &FrameRect) -> Option<FrameRect> {
    crate::ui::retained_host::host_contract::viewport_chrome_geometry::viewport_toolbar_frame(
        pane, content,
    )
}

pub(super) fn route_viewport_toolbar_hit<'a>(
    pane: &'a PaneData,
    toolbar: &FrameRect,
    x: f32,
    y: f32,
    surface_key: Option<&'a str>,
) -> Option<PanePointerRoute<'a>> {
    if !contains(toolbar, x, y) {
        return None;
    }
    Some(route_viewport_toolbar(pane, toolbar, x, y, surface_key))
}

use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};

use super::super::super::super::super::{geometry::contains, PanePointerRoute};
use super::super::super::toolbar::route_viewport_toolbar;

pub(super) fn viewport_toolbar_frame(pane: &PaneData, content: &FrameRect) -> Option<FrameRect> {
    if !matches!(pane.kind.as_str(), "Scene" | "Game") || !pane.show_toolbar {
        return None;
    }
    Some(FrameRect {
        x: content.x,
        y: content.y,
        width: content.width,
        height: 28.0_f32.min(content.height),
    })
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

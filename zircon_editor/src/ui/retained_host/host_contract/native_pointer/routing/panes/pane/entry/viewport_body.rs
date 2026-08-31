mod body;
mod model;
mod toolbar;

use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};

use self::body::viewport_body_frame_below_toolbar;
pub(super) use self::model::ViewportBodyRoute;
use self::toolbar::{route_viewport_toolbar_hit, viewport_toolbar_frame};

pub(super) fn viewport_body_route<'a>(
    pane: &'a PaneData,
    content: &FrameRect,
    x: f32,
    y: f32,
    surface_key: Option<&'a str>,
) -> ViewportBodyRoute<'a> {
    let Some(toolbar) = viewport_toolbar_frame(pane, content) else {
        return ViewportBodyRoute::content(content.clone());
    };
    if let Some(toolbar_route) = route_viewport_toolbar_hit(pane, &toolbar, x, y, surface_key) {
        return ViewportBodyRoute::toolbar(content.clone(), toolbar_route);
    }
    ViewportBodyRoute::content(viewport_body_frame_below_toolbar(content, toolbar.height))
}

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::frame_geometry::union_frame;

use super::super::super::super::routing::PanePointerRoute;

pub(super) fn hierarchy_scroll_damage(
    pointer: &PanePointerRoute,
    template_damage: Option<FrameRect>,
) -> FrameRect {
    template_damage
        .map(|template| union_frame(&template, &pointer.frame))
        .unwrap_or_else(|| pointer.frame.clone())
}

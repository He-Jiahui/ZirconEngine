use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};
use crate::ui::retained_host::host_contract::surface_hit_test;

use super::super::super::super::{PanePointerRoute, PanePointerTarget};
use super::super::super::mode::PaneRouteMode;

pub(super) fn route_template_node_hit(
    pane: &PaneData,
    body: &FrameRect,
    x: f32,
    y: f32,
    mode: PaneRouteMode,
) -> Option<PanePointerRoute> {
    if !mode.allows_template_hit_for_move(pane) {
        return None;
    }
    let hit = surface_hit_test::hit_test_pane_template_node(pane, body, x, y)?;
    Some(PanePointerRoute::new(
        PanePointerTarget::TemplateNode(hit),
        body,
        x,
        y,
    ))
}

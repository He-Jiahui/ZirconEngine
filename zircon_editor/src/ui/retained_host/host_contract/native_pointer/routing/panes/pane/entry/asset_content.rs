use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};
use crate::ui::workbench::asset_content_layout::ACTIVITY_CONTENT_PANEL_CONTROL_ID;

use super::super::super::super::{geometry::contains, PanePointerRoute, PanePointerTarget};

const ACTIVITY_ASSET_SURFACE_MODE: &str = "activity";

pub(super) fn route_activity_asset_content_hit(
    pane: &PaneData,
    body: &FrameRect,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute> {
    if pane.kind.as_str() != "Assets" {
        return None;
    }
    let panel = (0..pane.assets_activity.nodes.row_count())
        .filter_map(|row| pane.assets_activity.nodes.row_data(row))
        .find(|node| {
            node.control_id.rsplit('/').next() == Some(ACTIVITY_CONTENT_PANEL_CONTROL_ID)
        })?;
    let panel_frame = FrameRect {
        x: body.x + panel.frame.x,
        y: body.y + panel.frame.y,
        width: panel.frame.width.max(0.0),
        height: panel.frame.height.max(0.0),
    };
    if !contains(&panel_frame, x, y) {
        return None;
    }

    Some(PanePointerRoute::new(
        PanePointerTarget::AssetContent(ACTIVITY_ASSET_SURFACE_MODE.into()),
        &panel_frame,
        x,
        y,
    ))
}

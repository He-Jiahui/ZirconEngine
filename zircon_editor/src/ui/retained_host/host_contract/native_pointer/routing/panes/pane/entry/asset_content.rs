use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};
use crate::ui::workbench::asset_content_layout::{AssetContentPaintMetadata, AssetContentSurface};

use super::super::super::super::{
    geometry::contains, PaneAssetSurface, PanePointerRoute, PanePointerTarget,
};

pub(super) fn route_asset_content_hit(
    pane: &PaneData,
    body: &FrameRect,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute<'static>> {
    let (nodes, surface, surface_mode) = match pane.kind.as_str() {
        "Assets" => (
            &pane.assets_activity.nodes,
            AssetContentSurface::Activity,
            PaneAssetSurface::Activity,
        ),
        "AssetBrowser" => (
            &pane.asset_browser.nodes,
            AssetContentSurface::Browser,
            PaneAssetSurface::Browser,
        ),
        _ => return None,
    };
    let metadata = nodes.metadata::<AssetContentPaintMetadata>()?;
    if metadata.surface() != surface {
        return None;
    }
    let panel = metadata.content_panel()?;
    let panel_frame = FrameRect {
        x: body.x + panel.x,
        y: body.y + panel.y,
        width: panel.width.max(0.0),
        height: panel.height.max(0.0),
    };
    if !contains(&panel_frame, x, y) {
        return None;
    }

    Some(PanePointerRoute::new(
        PanePointerTarget::AssetContent(surface_mode),
        &panel_frame,
        x,
        y,
    ))
}

#[cfg(test)]
#[path = "tests/asset_content.rs"]
mod tests;

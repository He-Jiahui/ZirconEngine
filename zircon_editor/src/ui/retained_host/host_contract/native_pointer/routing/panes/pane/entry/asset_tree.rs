use crate::ui::retained_host::asset_pointer::asset_tree_viewport_y;
use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::{AssetContentPaintMetadata, AssetContentSurface};

use super::super::super::super::{
    geometry::contains, PaneAssetSurface, PanePointerRoute, PanePointerTarget,
};

pub(super) fn route_browser_asset_tree_hit(
    pane: &PaneData,
    body: &FrameRect,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute<'static>> {
    if pane.kind.as_str() != "AssetBrowser" {
        return None;
    }
    let panel = browser_sources_panel_frame(&pane.asset_browser.nodes, body)?;
    contains(&panel, x, y).then(|| {
        PanePointerRoute::new(
            PanePointerTarget::AssetTree(PaneAssetSurface::Browser),
            &panel,
            x,
            y,
        )
    })
}

fn browser_sources_panel_frame(
    nodes: &ModelRc<crate::ui::retained_host::host_contract::data::TemplatePaneNodeData>,
    body: &FrameRect,
) -> Option<FrameRect> {
    let metadata = nodes.metadata::<AssetContentPaintMetadata>()?;
    if metadata.surface() != AssetContentSurface::Browser {
        return None;
    }
    let viewport = metadata.browser_source_tree_viewport()?;
    let header_height = asset_tree_viewport_y();
    Some(FrameRect {
        x: body.x + viewport.x,
        y: body.y + viewport.y - header_height,
        width: viewport.width,
        height: viewport.height + header_height,
    })
}

#[cfg(test)]
#[path = "tests/asset_tree.rs"]
mod tests;

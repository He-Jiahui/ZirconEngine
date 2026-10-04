use crate::ui::retained_host::asset_pointer::asset_reference_viewport_y;
use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::{
    ActivityAssetReferenceListKind, AssetContentPaintMetadata, AssetContentSurface,
    BrowserAssetReferenceListKind,
};

use super::super::super::super::{
    geometry::contains, PaneAssetReferenceList, PaneAssetSurface, PanePointerRoute,
    PanePointerTarget,
};

pub(super) fn route_asset_reference_hit(
    pane: &PaneData,
    body: &FrameRect,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute<'static>> {
    match pane.kind.as_str() {
        "Assets" => route_activity_asset_reference_hit(pane, body, x, y),
        "AssetBrowser" => route_browser_asset_reference_hit(pane, body, x, y),
        _ => None,
    }
}

fn route_activity_asset_reference_hit(
    pane: &PaneData,
    body: &FrameRect,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute<'static>> {
    for (list_kind, callback_kind) in [
        (
            ActivityAssetReferenceListKind::References,
            PaneAssetReferenceList::References,
        ),
        (
            ActivityAssetReferenceListKind::UsedBy,
            PaneAssetReferenceList::UsedBy,
        ),
    ] {
        let Some(panel) =
            activity_reference_panel_frame(&pane.assets_activity.nodes, body, list_kind)
        else {
            continue;
        };
        if contains(&panel, x, y) {
            return Some(PanePointerRoute::new(
                PanePointerTarget::AssetReference(PaneAssetSurface::Activity, callback_kind),
                &panel,
                x,
                y,
            ));
        }
    }
    None
}

fn route_browser_asset_reference_hit(
    pane: &PaneData,
    body: &FrameRect,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute<'static>> {
    for (list_kind, callback_kind) in [
        (
            BrowserAssetReferenceListKind::References,
            PaneAssetReferenceList::References,
        ),
        (
            BrowserAssetReferenceListKind::UsedBy,
            PaneAssetReferenceList::UsedBy,
        ),
    ] {
        let Some(panel) = browser_reference_panel_frame(&pane.asset_browser.nodes, body, list_kind)
        else {
            continue;
        };
        if contains(&panel, x, y) {
            return Some(PanePointerRoute::new(
                PanePointerTarget::AssetReference(PaneAssetSurface::Browser, callback_kind),
                &panel,
                x,
                y,
            ));
        }
    }
    None
}

fn browser_reference_panel_frame(
    nodes: &ModelRc<crate::ui::retained_host::host_contract::data::TemplatePaneNodeData>,
    body: &FrameRect,
    list_kind: BrowserAssetReferenceListKind,
) -> Option<FrameRect> {
    let metadata = nodes.metadata::<AssetContentPaintMetadata>()?;
    if metadata.surface() != AssetContentSurface::Browser {
        return None;
    }
    let viewport = metadata.browser_reference_viewport(list_kind)?;
    if viewport.width <= 0.0 || viewport.height <= 0.0 {
        return None;
    }
    let header_height = asset_reference_viewport_y();
    Some(FrameRect {
        x: body.x + viewport.x,
        y: body.y + viewport.y - header_height,
        width: viewport.width,
        height: viewport.height + header_height,
    })
}

fn activity_reference_panel_frame(
    nodes: &ModelRc<crate::ui::retained_host::host_contract::data::TemplatePaneNodeData>,
    body: &FrameRect,
    list_kind: ActivityAssetReferenceListKind,
) -> Option<FrameRect> {
    let metadata = nodes.metadata::<AssetContentPaintMetadata>()?;
    if metadata.surface() != AssetContentSurface::Activity {
        return None;
    }
    let viewport = metadata.activity_reference_viewport(list_kind)?;
    if viewport.width <= 0.0 || viewport.height <= 0.0 {
        return None;
    }
    let header_height = asset_reference_viewport_y();
    Some(FrameRect {
        x: body.x + viewport.x,
        y: body.y + viewport.y - header_height,
        width: viewport.width,
        height: viewport.height + header_height,
    })
}

#[cfg(test)]
#[path = "tests/asset_reference.rs"]
mod tests;

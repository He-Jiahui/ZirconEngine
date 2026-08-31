mod asset;
mod geometry;
mod paint;
mod style;

use crate::ui::retained_host::hierarchy_pointer::{hierarchy_content_height, HierarchyRowMetrics};
use crate::ui::workbench::asset_content_layout::{
    AssetContentPaintMetadata, AssetContentScrollbarKind, AssetContentSurface,
};

use super::super::super::data::{
    FrameRect, HostPaneInteractionStateData, PaneData, TemplatePaneNodeData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::intersect;
use asset::{asset_scrollbar_content_extent, asset_scrollbar_viewport};
pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer) use paint::draw_vertical_scrollbar;

pub(super) fn draw_hierarchy_scrollbar(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    viewport: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
    row_metrics: HierarchyRowMetrics,
) -> bool {
    let node_count = pane.hierarchy.hierarchy_nodes.row_count();
    if node_count == 0 {
        return false;
    }
    draw_vertical_scrollbar(
        frame,
        viewport,
        clip,
        interaction.hierarchy_scroll_px,
        hierarchy_content_height(node_count, row_metrics),
        interaction.hovered_hierarchy_index >= 0,
    )
}

pub(super) fn draw_activity_asset_scrollbars(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    draw_asset_scrollbars(
        frame,
        &pane.assets_activity.nodes,
        AssetContentSurface::Activity,
        body,
        clip,
        interaction,
    )
}

pub(super) fn draw_browser_asset_scrollbars(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    draw_asset_scrollbars(
        frame,
        &pane.asset_browser.nodes,
        AssetContentSurface::Browser,
        body,
        clip,
        interaction,
    )
}

fn draw_asset_scrollbars(
    frame: &mut HostRgbaFrame,
    nodes: &crate::ui::retained_host::primitives::ModelRc<TemplatePaneNodeData>,
    expected_surface: AssetContentSurface,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    let Some(metadata) = nodes.metadata::<AssetContentPaintMetadata>() else {
        return false;
    };
    if metadata.surface() != expected_surface {
        return false;
    }
    let mut painted = false;
    for descriptor in metadata.scrollbar_descriptors() {
        let Some(viewport) = metadata.scrollbar_viewport(*descriptor) else {
            continue;
        };
        let viewport = asset_scrollbar_viewport(viewport, body);
        if intersect(&viewport, clip).is_none() {
            continue;
        }
        let (scroll_px, active) =
            asset_scrollbar_interaction(expected_surface, descriptor.kind(), interaction);
        let current = draw_vertical_scrollbar(
            frame,
            &viewport,
            clip,
            scroll_px,
            asset_scrollbar_content_extent(metadata.scrollbar_extent(*descriptor)),
            active,
        );
        painted = current || painted;
    }
    painted
}

fn asset_scrollbar_interaction(
    surface: AssetContentSurface,
    kind: AssetContentScrollbarKind,
    interaction: &HostPaneInteractionStateData,
) -> (f32, bool) {
    match (surface, kind) {
        (AssetContentSurface::Activity, AssetContentScrollbarKind::Tree) => (
            interaction.activity_asset_tree_scroll_px,
            interaction.activity_asset_tree_hovered_index >= 0,
        ),
        (AssetContentSurface::Activity, AssetContentScrollbarKind::Content) => (
            interaction.activity_asset_content_scroll_px,
            interaction.activity_asset_content_hovered_index >= 0,
        ),
        (AssetContentSurface::Activity, AssetContentScrollbarKind::References) => (
            interaction.activity_asset_references_scroll_px,
            interaction.activity_asset_references_hovered_index >= 0,
        ),
        (AssetContentSurface::Activity, AssetContentScrollbarKind::UsedBy) => (
            interaction.activity_asset_used_by_scroll_px,
            interaction.activity_asset_used_by_hovered_index >= 0,
        ),
        (AssetContentSurface::Browser, AssetContentScrollbarKind::Tree) => (
            interaction.browser_asset_tree_scroll_px,
            interaction.browser_asset_tree_hovered_index >= 0,
        ),
        (AssetContentSurface::Browser, AssetContentScrollbarKind::Content) => (
            interaction.browser_asset_content_scroll_px,
            interaction.browser_asset_content_hovered_index >= 0,
        ),
        (AssetContentSurface::Browser, AssetContentScrollbarKind::References) => (
            interaction.browser_asset_references_scroll_px,
            interaction.browser_asset_references_hovered_index >= 0,
        ),
        (AssetContentSurface::Browser, AssetContentScrollbarKind::UsedBy) => (
            interaction.browser_asset_used_by_scroll_px,
            interaction.browser_asset_used_by_hovered_index >= 0,
        ),
    }
}

#[cfg(test)]
pub(crate) fn paint_scrollbar_component_for_test(width: u32, height: u32) -> Vec<u8> {
    paint::paint_scrollbar_component_for_test(width, height)
}

#[cfg(test)]
mod tests;

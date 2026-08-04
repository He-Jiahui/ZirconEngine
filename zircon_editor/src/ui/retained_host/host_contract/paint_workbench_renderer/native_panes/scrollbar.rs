mod asset;
mod geometry;
mod paint;
mod style;

use crate::ui::retained_host::hierarchy_pointer::{
    current_hierarchy_row_metrics, hierarchy_content_height,
};
use crate::ui::workbench::asset_content_layout::{
    ActivityAssetReferenceListKind, BrowserAssetReferenceListKind,
};

use super::super::super::data::{
    FrameRect, HostPaneInteractionStateData, PaneData, TemplatePaneNodeData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::hierarchy::hierarchy_viewport_frame;
use asset::{
    activity_asset_content_viewport_and_extent, activity_asset_reference_viewport_and_row_count,
    asset_tree_row_count, asset_tree_viewport_frame, browser_asset_content_viewport_and_extent,
    browser_asset_reference_viewport_and_row_count, browser_asset_tree_viewport_frame,
};
pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer) use paint::draw_vertical_scrollbar;

const ACTIVITY_ASSET_TREE_ROW_CONTROL: &str = "AssetsActivityTreeRowPanel";
const BROWSER_ASSET_TREE_ROW_CONTROL: &str = "AssetBrowserSourcesRowPanel";

pub(super) fn draw_hierarchy_scrollbar(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    let node_count = pane.hierarchy.hierarchy_nodes.row_count();
    if node_count == 0 {
        return false;
    }
    let viewport = hierarchy_viewport_frame(pane, body);
    draw_vertical_scrollbar(
        frame,
        &viewport,
        clip,
        interaction.hierarchy_scroll_px,
        hierarchy_content_height(node_count, current_hierarchy_row_metrics()),
        interaction.hovered_hierarchy_index >= 0,
    )
}

pub(super) fn draw_activity_asset_tree_scrollbar(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    draw_asset_tree_scrollbar(
        frame,
        &pane.assets_activity.nodes,
        body,
        clip,
        ACTIVITY_ASSET_TREE_ROW_CONTROL,
        interaction.activity_asset_tree_scroll_px,
        interaction.activity_asset_tree_hovered_index >= 0,
    )
}

pub(super) fn draw_browser_asset_tree_scrollbar(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    let nodes = &pane.asset_browser.nodes;
    let row_count = asset_tree_row_count(nodes, BROWSER_ASSET_TREE_ROW_CONTROL);
    let Some(viewport) = browser_asset_tree_viewport_frame(nodes, body) else {
        return false;
    };
    draw_vertical_scrollbar(
        frame,
        &viewport,
        clip,
        interaction.browser_asset_tree_scroll_px,
        crate::ui::retained_host::asset_pointer::asset_tree_content_height(row_count),
        interaction.browser_asset_tree_hovered_index >= 0,
    )
}

pub(super) fn draw_activity_asset_content_scrollbar(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    let Some((viewport, content_extent)) =
        activity_asset_content_viewport_and_extent(&pane.assets_activity.nodes, body)
    else {
        return false;
    };
    draw_vertical_scrollbar(
        frame,
        &viewport,
        clip,
        interaction.activity_asset_content_scroll_px,
        content_extent,
        interaction.activity_asset_content_hovered_index >= 0,
    )
}

pub(super) fn draw_browser_asset_content_scrollbar(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    let Some((viewport, content_extent)) =
        browser_asset_content_viewport_and_extent(&pane.asset_browser.nodes, body)
    else {
        return false;
    };
    draw_vertical_scrollbar(
        frame,
        &viewport,
        clip,
        interaction.browser_asset_content_scroll_px,
        content_extent,
        interaction.browser_asset_content_hovered_index >= 0,
    )
}

pub(super) fn draw_activity_asset_reference_scrollbars(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    let references = draw_activity_asset_reference_scrollbar(
        frame,
        pane,
        body,
        clip,
        ActivityAssetReferenceListKind::References,
        interaction.activity_asset_references_scroll_px,
        interaction.activity_asset_references_hovered_index >= 0,
    );
    let used_by = draw_activity_asset_reference_scrollbar(
        frame,
        pane,
        body,
        clip,
        ActivityAssetReferenceListKind::UsedBy,
        interaction.activity_asset_used_by_scroll_px,
        interaction.activity_asset_used_by_hovered_index >= 0,
    );
    references || used_by
}

fn draw_activity_asset_reference_scrollbar(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    list_kind: ActivityAssetReferenceListKind,
    scroll_px: f32,
    active: bool,
) -> bool {
    let Some((viewport, row_count)) = activity_asset_reference_viewport_and_row_count(
        &pane.assets_activity.nodes,
        body,
        list_kind,
    ) else {
        return false;
    };
    draw_vertical_scrollbar(
        frame,
        &viewport,
        clip,
        scroll_px,
        crate::ui::retained_host::asset_pointer::asset_reference_content_height(row_count),
        active,
    )
}

pub(super) fn draw_browser_asset_reference_scrollbars(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    let references = draw_browser_asset_reference_scrollbar(
        frame,
        pane,
        body,
        clip,
        BrowserAssetReferenceListKind::References,
        interaction.browser_asset_references_scroll_px,
        interaction.browser_asset_references_hovered_index >= 0,
    );
    let used_by = draw_browser_asset_reference_scrollbar(
        frame,
        pane,
        body,
        clip,
        BrowserAssetReferenceListKind::UsedBy,
        interaction.browser_asset_used_by_scroll_px,
        interaction.browser_asset_used_by_hovered_index >= 0,
    );
    references || used_by
}

fn draw_browser_asset_reference_scrollbar(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    list_kind: BrowserAssetReferenceListKind,
    scroll_px: f32,
    active: bool,
) -> bool {
    let Some((viewport, row_count)) =
        browser_asset_reference_viewport_and_row_count(&pane.asset_browser.nodes, body, list_kind)
    else {
        return false;
    };
    draw_vertical_scrollbar(
        frame,
        &viewport,
        clip,
        scroll_px,
        crate::ui::retained_host::asset_pointer::asset_reference_content_height(row_count),
        active,
    )
}

fn draw_asset_tree_scrollbar(
    frame: &mut HostRgbaFrame,
    nodes: &crate::ui::retained_host::primitives::ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
    clip: &FrameRect,
    row_control_id: &str,
    scroll_px: f32,
    active: bool,
) -> bool {
    let row_count = asset_tree_row_count(nodes, row_control_id);
    if row_count == 0 {
        return false;
    }
    draw_vertical_scrollbar(
        frame,
        &asset_tree_viewport_frame(body),
        clip,
        scroll_px,
        crate::ui::retained_host::asset_pointer::asset_tree_content_height(row_count),
        active,
    )
}

#[cfg(test)]
pub(crate) fn paint_scrollbar_component_for_test(width: u32, height: u32) -> Vec<u8> {
    paint::paint_scrollbar_component_for_test(width, height)
}

#[cfg(test)]
mod tests;

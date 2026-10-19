use std::collections::HashMap;

use zircon_runtime_interface::ui::design_tokens::{EditorDensityTokens, EditorTypographyTokens};

use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::workbench::asset_content_layout::{
    activity_compact_name, activity_row_geometry, ActivityAssetVirtualization,
    AssetContentLayoutMetrics, AssetContentRect, AssetContentSurfaceProfile,
    ACTIVITY_CONTENT_PANEL_CONTROL_ID,
};
use crate::ui::workbench::snapshot::{AssetViewMode, AssetWorkspaceSnapshot};

use super::content_nodes::{
    item_badge_control_id, item_meta_control_id, item_name_control_id, item_row_control_id,
    item_type_control_id, EMPTY_CONTROL_ID,
};

struct ActivityContentNodeIndex {
    by_control_id: HashMap<SharedString, usize>,
}

impl ActivityContentNodeIndex {
    fn from_nodes(nodes: &[ViewTemplateNodeData]) -> Self {
        let mut by_control_id = HashMap::with_capacity(nodes.len());
        for (index, node) in nodes.iter().enumerate() {
            by_control_id
                .entry(node.control_id.clone())
                .or_insert(index);
        }
        Self { by_control_id }
    }

    fn index_of(&self, control_id: &str) -> Option<usize> {
        self.by_control_id.get(control_id).copied()
    }
}

pub(super) fn apply_assets_activity_content_layout(
    nodes: &mut [ViewTemplateNodeData],
    snapshot: &AssetWorkspaceSnapshot,
    virtual_rows: &ActivityAssetVirtualization,
) {
    let node_index = ActivityContentNodeIndex::from_nodes(nodes);
    let Some(panel_index) = node_index.index_of(ACTIVITY_CONTENT_PANEL_CONTROL_ID) else {
        return;
    };
    let panel = nodes[panel_index].frame.clone();
    let metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Activity,
        snapshot.view_mode,
    );
    nodes[panel_index].value_number = metrics.list_height(
        snapshot.visible_folders.len(),
        snapshot.visible_assets.len(),
    );
    if snapshot.visible_folders.is_empty() && snapshot.visible_assets.is_empty() {
        layout_empty_state(nodes, &node_index, &panel, metrics);
        return;
    }

    for index in 0..virtual_rows.slot_count {
        let Some(binding) = virtual_rows.binding(0.0, index) else {
            continue;
        };
        layout_content_row(
            nodes,
            &node_index,
            &panel,
            metrics,
            snapshot.view_mode,
            binding.top,
            binding.height,
            &item_row_control_id(index),
            &item_badge_control_id(index),
            &item_type_control_id(index),
            &item_name_control_id(index),
            &item_meta_control_id(index),
        );
        compact_item_name(nodes, &node_index, &item_name_control_id(index));
    }
}

#[allow(clippy::too_many_arguments)]
fn layout_content_row(
    nodes: &mut [ViewTemplateNodeData],
    index: &ActivityContentNodeIndex,
    panel: &ViewTemplateFrameData,
    metrics: AssetContentLayoutMetrics,
    view_mode: AssetViewMode,
    y: f32,
    height: f32,
    row_control_id: &str,
    badge_control_id: &str,
    type_control_id: &str,
    name_control_id: &str,
    meta_control_id: &str,
) {
    let row_width = metrics.row_width(panel.width);
    if row_width <= 0.0 {
        hide_controls(
            nodes,
            index,
            [
                row_control_id,
                badge_control_id,
                type_control_id,
                name_control_id,
                meta_control_id,
            ],
            panel.x + panel.width,
            panel.y + panel.height,
        );
        return;
    }

    let meta_text = index
        .index_of(meta_control_id)
        .and_then(|index| nodes.get(index))
        .map(|node| node.text.as_str())
        .unwrap_or("");
    let geometry = activity_row_geometry(
        AssetContentRect {
            x: panel.x,
            y: panel.y,
            width: panel.width,
            height: panel.height,
        },
        metrics,
        view_mode,
        y,
        height,
        meta_text,
    );
    for (control_id, frame) in [
        (row_control_id, geometry.row),
        (badge_control_id, geometry.badge),
        (type_control_id, geometry.badge),
        (name_control_id, geometry.name),
        (meta_control_id, geometry.meta),
    ] {
        set_frame(
            nodes,
            index,
            control_id,
            frame.x,
            frame.y,
            frame.width,
            frame.height,
        );
    }
}

fn compact_item_name(
    nodes: &mut [ViewTemplateNodeData],
    index: &ActivityContentNodeIndex,
    control_id: &str,
) {
    let Some(node) = index
        .index_of(control_id)
        .and_then(|index| nodes.get_mut(index))
    else {
        return;
    };
    if node.frame.width <= 0.0 {
        return;
    }
    node.text = activity_compact_name(
        node.text.as_str(),
        node.value_text.as_str(),
        node.frame.width,
    )
    .into();
}

fn layout_empty_state(
    nodes: &mut [ViewTemplateNodeData],
    index: &ActivityContentNodeIndex,
    panel: &ViewTemplateFrameData,
    metrics: AssetContentLayoutMetrics,
) {
    let density = EditorDensityTokens::workbench_dense();
    let height = density.row_height;
    set_frame(
        nodes,
        index,
        EMPTY_CONTROL_ID,
        panel.x + metrics.row_x,
        panel.y + metrics.first_row_y(),
        metrics.row_width(panel.width),
        height.min(panel.height.max(0.0)),
    );
}

fn set_frame(
    nodes: &mut [ViewTemplateNodeData],
    index: &ActivityContentNodeIndex,
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    if let Some(node) = index
        .index_of(control_id)
        .and_then(|index| nodes.get_mut(index))
    {
        node.frame = ViewTemplateFrameData {
            x,
            y,
            width: width.max(0.0),
            height: height.max(0.0),
        };
    }
}

fn hide_controls<'a>(
    nodes: &mut [ViewTemplateNodeData],
    index: &ActivityContentNodeIndex,
    control_ids: impl IntoIterator<Item = &'a str>,
    x: f32,
    y: f32,
) {
    for control_id in control_ids {
        set_frame(nodes, index, control_id, x, y, 0.0, 0.0);
    }
}

#[cfg(test)]
#[path = "../content_layout/tests/indexed_lookup.rs"]
mod indexed_lookup_tests;

use std::collections::HashMap;

use slint::SharedString;
use zircon_runtime_interface::ui::design_tokens::{EditorDensityTokens, EditorTypographyTokens};

use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::workbench::asset_content_layout::{
    compact_file_like_display_name, AssetContentLayoutMetrics, AssetContentSurfaceProfile,
    RuntimeFileNameCompaction, ACTIVITY_CONTENT_PANEL_CONTROL_ID,
};
use crate::ui::workbench::snapshot::{AssetViewMode, AssetWorkspaceSnapshot};

use super::content_nodes::{
    folder_badge_control_id, folder_meta_control_id, folder_name_control_id, folder_row_control_id,
    folder_type_control_id, item_badge_control_id, item_meta_control_id, item_name_control_id,
    item_row_control_id, item_type_control_id, EMPTY_CONTROL_ID,
};

const ACTIVITY_NAME_MIN_PREFIX_CHARS: usize = 4;
const ACTIVITY_NAME_MIN_TAIL_STEM_CHARS: usize = 3;
const ACTIVITY_NAME_PREFERRED_TAIL_STEM_CHARS: usize = 6;
const META_MIN_WIDTH_ROW_FRACTION: f32 = 0.75;

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

    let mut y = panel.y + metrics.first_row_y();
    for index in 0..snapshot.visible_folders.len() {
        layout_content_row(
            nodes,
            &node_index,
            &panel,
            metrics,
            snapshot.view_mode,
            y,
            metrics.folder_height,
            &folder_row_control_id(index),
            &folder_badge_control_id(index),
            &folder_type_control_id(index),
            &folder_name_control_id(index),
            &folder_meta_control_id(index),
        );
        y += metrics.folder_height + metrics.row_gap;
    }
    for index in 0..snapshot.visible_assets.len() {
        let name_control_id = item_name_control_id(index);
        layout_content_row(
            nodes,
            &node_index,
            &panel,
            metrics,
            snapshot.view_mode,
            y,
            metrics.item_height,
            &item_row_control_id(index),
            &item_badge_control_id(index),
            &item_type_control_id(index),
            &name_control_id,
            &item_meta_control_id(index),
        );
        compact_item_name(nodes, &node_index, &name_control_id);
        y += metrics.item_height + metrics.row_gap;
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
    let row_x = panel.x + metrics.row_x;
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

    set_frame(nodes, index, row_control_id, row_x, y, row_width, height);
    let density = EditorDensityTokens::workbench_dense();
    let inner_gap = density.gap_small;
    let badge_extent = match view_mode {
        AssetViewMode::List => density.row_height,
        AssetViewMode::Thumbnail => (height - density.gap_medium * 2.0).max(density.row_height),
    };
    let badge_x = row_x + inner_gap;
    let badge_y = y + (height - badge_extent) * 0.5;
    set_frame(
        nodes,
        index,
        badge_control_id,
        badge_x,
        badge_y,
        badge_extent,
        badge_extent,
    );
    set_frame(
        nodes,
        index,
        type_control_id,
        badge_x,
        badge_y,
        badge_extent,
        badge_extent,
    );

    let typography = EditorTypographyTokens::workbench_default();
    let meta_width = index
        .index_of(meta_control_id)
        .and_then(|index| nodes.get(index))
        .map(|node| {
            measure_runtime_text_width(node.text.as_str(), typography.caption_size)
                + density.gap_medium
        })
        .unwrap_or(density.row_height)
        .max(density.row_height * META_MIN_WIDTH_ROW_FRACTION);
    let meta_x = row_x + row_width - inner_gap - meta_width;
    let name_x = badge_x + badge_extent + density.gap_medium;
    let name_width = (meta_x - density.gap_small - name_x).max(0.0);
    let label_height = typography.body_size * typography.line_height;
    let label_y = y + (height - label_height) * 0.5;
    set_frame(
        nodes,
        index,
        name_control_id,
        name_x,
        label_y,
        name_width,
        label_height,
    );
    set_frame(
        nodes,
        index,
        meta_control_id,
        meta_x,
        label_y,
        meta_width,
        label_height,
    );
}

fn compact_item_name(
    nodes: &mut [ViewTemplateNodeData],
    index: &ActivityContentNodeIndex,
    control_id: &str,
) {
    let typography = EditorTypographyTokens::workbench_default();
    let Some(node) = index
        .index_of(control_id)
        .and_then(|index| nodes.get_mut(index))
    else {
        return;
    };
    if node.frame.width <= 0.0 {
        return;
    }
    node.text = compact_file_like_display_name(
        node.text.as_str(),
        node.value_text.as_str(),
        RuntimeFileNameCompaction {
            max_width: node.frame.width,
            font_size: typography.body_size,
            min_prefix_chars: ACTIVITY_NAME_MIN_PREFIX_CHARS,
            min_tail_stem_chars: ACTIVITY_NAME_MIN_TAIL_STEM_CHARS,
            preferred_tail_stem_chars: ACTIVITY_NAME_PREFERRED_TAIL_STEM_CHARS,
        },
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
mod indexed_lookup_tests;

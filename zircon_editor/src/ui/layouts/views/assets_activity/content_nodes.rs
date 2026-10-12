use zircon_runtime_interface::ui::design_tokens::{EditorControlTokens, EditorTypographyTokens};

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::workbench::asset_content_layout::{
    ActivityAssetVirtualization, ACTIVITY_CONTENT_EMPTY_CONTROL_ID, ACTIVITY_CONTENT_ITEM_PREFIX,
};
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;

pub(super) const EMPTY_CONTROL_ID: &str = ACTIVITY_CONTENT_EMPTY_CONTROL_ID;
const CONTENT_ROW_LAYER: i32 = 20;
const CONTENT_BADGE_LAYER: i32 = 21;
const CONTENT_LABEL_LAYER: i32 = 22;
const CONTENT_NODES_PER_ROW: usize = 5;

pub(super) fn append_assets_activity_content_nodes(
    nodes: &mut Vec<ViewTemplateNodeData>,
    snapshot: &AssetWorkspaceSnapshot,
    virtual_rows: &ActivityAssetVirtualization,
) {
    let slot_count = virtual_rows.slot_count;
    nodes.reserve(assets_activity_content_node_count(0, slot_count));
    if snapshot.visible_folders.is_empty() && snapshot.visible_assets.is_empty() {
        nodes.push(empty_state_node());
        return;
    }
    // Physical controls identify slots. Logical folder/asset identity is generation-bound metadata.
    for index in 0..slot_count {
        let Some(binding) = virtual_rows.binding(0.0, index) else {
            continue;
        };
        nodes.push(row_panel(item_row_control_id(index), binding.selected));
        nodes.push(type_badge(item_badge_control_id(index)));
        nodes.push(label_node(
            item_type_control_id(index),
            binding.badge().to_string(),
            "accent",
            typography().caption_size,
            typography().strong_weight as i32,
        ));
        let mut name = label_node(
            item_name_control_id(index),
            binding.name().to_string(),
            "",
            typography().body_size,
            typography().body_weight as i32,
        );
        name.value_text = binding.extension().into();
        nodes.push(name);
        nodes.push(label_node(
            item_meta_control_id(index),
            binding.meta(),
            binding.meta_tone(),
            typography().caption_size,
            typography().body_weight as i32,
        ));
    }
}

fn assets_activity_content_node_count(folder_count: usize, asset_count: usize) -> usize {
    let row_count = folder_count.saturating_add(asset_count);
    if row_count == 0 {
        1
    } else {
        row_count.saturating_mul(CONTENT_NODES_PER_ROW)
    }
}

fn empty_state_node() -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: "assets_activity.content.empty".into(),
        control_id: EMPTY_CONTROL_ID.into(),
        role: "Label".into(),
        text: "No assets in this folder".into(),
        text_tone: "muted".into(),
        text_align: "center".into(),
        overflow: "elide".into(),
        font_size: typography().body_size,
        font_weight: typography().body_weight as i32,
        z_index: CONTENT_ROW_LAYER,
        options: model_rc(Vec::<SharedString>::new()),
        ..ViewTemplateNodeData::default()
    }
}

fn row_panel(control_id: String, selected: bool) -> ViewTemplateNodeData {
    let controls = EditorControlTokens::workbench_dense();
    ViewTemplateNodeData {
        node_id: format!("assets_activity.content.{control_id}").into(),
        control_id: control_id.into(),
        role: "Panel".into(),
        component_role: "workbench-list-row".into(),
        surface_variant: "asset-thumbnail-name-area".into(),
        corner_radius: controls.small_radius,
        border_width: if selected { controls.border_width } else { 0.0 },
        selected,
        z_index: CONTENT_ROW_LAYER,
        ..ViewTemplateNodeData::default()
    }
}

fn type_badge(control_id: String) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: format!("assets_activity.content.{control_id}").into(),
        control_id: control_id.into(),
        role: "Panel".into(),
        surface_variant: "asset-type-badge".into(),
        corner_radius: EditorControlTokens::workbench_dense().small_radius,
        z_index: CONTENT_BADGE_LAYER,
        ..ViewTemplateNodeData::default()
    }
}

fn label_node(
    control_id: String,
    text: String,
    text_tone: &str,
    font_size: f32,
    font_weight: i32,
) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: format!("assets_activity.content.{control_id}").into(),
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        text_tone: text_tone.into(),
        overflow: "elide".into(),
        font_size,
        font_weight,
        z_index: CONTENT_LABEL_LAYER,
        options: model_rc(Vec::<SharedString>::new()),
        ..ViewTemplateNodeData::default()
    }
}

fn typography() -> EditorTypographyTokens {
    EditorTypographyTokens::workbench_default()
}

pub(super) fn item_row_control_id(index: usize) -> String {
    format!("{ACTIVITY_CONTENT_ITEM_PREFIX}Row{index:02}")
}

pub(super) fn item_badge_control_id(index: usize) -> String {
    format!("{ACTIVITY_CONTENT_ITEM_PREFIX}Badge{index:02}")
}

pub(super) fn item_type_control_id(index: usize) -> String {
    format!("{ACTIVITY_CONTENT_ITEM_PREFIX}Type{index:02}")
}

pub(super) fn item_name_control_id(index: usize) -> String {
    format!("{ACTIVITY_CONTENT_ITEM_PREFIX}Name{index:02}")
}

pub(super) fn item_meta_control_id(index: usize) -> String {
    format!("{ACTIVITY_CONTENT_ITEM_PREFIX}Meta{index:02}")
}

#[cfg(test)]
#[path = "../content_nodes/tests/reserve.rs"]
mod reserve_tests;

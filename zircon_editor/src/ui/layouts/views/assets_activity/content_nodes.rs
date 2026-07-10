use zircon_runtime_interface::ui::design_tokens::{EditorControlTokens, EditorTypographyTokens};

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::workbench::asset_content_layout::{
    resource_kind_badge_code, ACTIVITY_CONTENT_EMPTY_CONTROL_ID, ACTIVITY_CONTENT_FOLDER_PREFIX,
    ACTIVITY_CONTENT_ITEM_PREFIX,
};
use crate::ui::workbench::snapshot::{
    AssetFolderSnapshot, AssetItemSnapshot, AssetWorkspaceSnapshot,
};

pub(super) const EMPTY_CONTROL_ID: &str = ACTIVITY_CONTENT_EMPTY_CONTROL_ID;
const CONTENT_ROW_LAYER: i32 = 20;
const CONTENT_BADGE_LAYER: i32 = 21;
const CONTENT_LABEL_LAYER: i32 = 22;

pub(super) fn append_assets_activity_content_nodes(
    nodes: &mut Vec<ViewTemplateNodeData>,
    snapshot: &AssetWorkspaceSnapshot,
) {
    if snapshot.visible_folders.is_empty() && snapshot.visible_assets.is_empty() {
        nodes.push(empty_state_node());
        return;
    }

    for (index, folder) in snapshot.visible_folders.iter().enumerate() {
        let selected = folder.selected
            || snapshot.selected_folder_id.as_deref() == Some(folder.folder_id.as_str());
        nodes.push(row_panel(folder_row_control_id(index), selected));
        nodes.push(type_badge(folder_badge_control_id(index)));
        nodes.push(label_node(
            folder_type_control_id(index),
            "DIR".to_string(),
            "accent",
            typography().caption_size,
            typography().strong_weight as i32,
        ));
        nodes.push(label_node(
            folder_name_control_id(index),
            folder.display_name.clone(),
            "",
            typography().body_size,
            typography().body_weight as i32,
        ));
        nodes.push(label_node(
            folder_meta_control_id(index),
            folder_count_text(folder),
            "muted",
            typography().caption_size,
            typography().body_weight as i32,
        ));
    }

    for (index, asset) in snapshot.visible_assets.iter().enumerate() {
        let selected =
            asset.selected || snapshot.selected_asset_uuid.as_deref() == Some(asset.uuid.as_str());
        nodes.push(row_panel(item_row_control_id(index), selected));
        nodes.push(type_badge(item_badge_control_id(index)));
        nodes.push(label_node(
            item_type_control_id(index),
            resource_kind_badge_code(asset.kind).to_string(),
            "accent",
            typography().caption_size,
            typography().strong_weight as i32,
        ));
        let mut name = label_node(
            item_name_control_id(index),
            asset.display_name.clone(),
            "",
            typography().body_size,
            typography().body_weight as i32,
        );
        name.value_text = asset.extension.clone().into();
        nodes.push(name);
        nodes.push(label_node(
            item_meta_control_id(index),
            asset_meta_text(asset),
            if asset.diagnostics.is_empty() {
                "muted"
            } else {
                "danger"
            },
            typography().caption_size,
            typography().body_weight as i32,
        ));
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

fn folder_count_text(folder: &AssetFolderSnapshot) -> String {
    folder.recursive_asset_count.to_string()
}

fn asset_meta_text(asset: &AssetItemSnapshot) -> String {
    if !asset.diagnostics.is_empty() {
        return "!".to_string();
    }
    match asset.resource_revision {
        Some(revision) => format!("r{revision}"),
        None if asset.dirty => "*".to_string(),
        None => "new".to_string(),
    }
}

pub(super) fn folder_row_control_id(index: usize) -> String {
    format!("{ACTIVITY_CONTENT_FOLDER_PREFIX}Row{index:02}")
}

pub(super) fn folder_badge_control_id(index: usize) -> String {
    format!("{ACTIVITY_CONTENT_FOLDER_PREFIX}Badge{index:02}")
}

pub(super) fn folder_type_control_id(index: usize) -> String {
    format!("{ACTIVITY_CONTENT_FOLDER_PREFIX}Type{index:02}")
}

pub(super) fn folder_name_control_id(index: usize) -> String {
    format!("{ACTIVITY_CONTENT_FOLDER_PREFIX}Name{index:02}")
}

pub(super) fn folder_meta_control_id(index: usize) -> String {
    format!("{ACTIVITY_CONTENT_FOLDER_PREFIX}Meta{index:02}")
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

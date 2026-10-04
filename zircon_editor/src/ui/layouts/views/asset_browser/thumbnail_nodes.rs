use super::labels::asset_state_label;
use super::name_lines::{split_display_name_lines, RuntimeNameLineSplit};
use zircon_runtime_interface::ui::design_tokens::{EditorControlTokens, EditorTypographyTokens};

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host::{measure_runtime_text_width, primitives::SharedString};
use crate::ui::workbench::asset_content_layout::{
    compact_thumbnail_file_name_to_width as compact_thumbnail_name_to_width, AssetBrowserPaintItem,
    AssetBrowserThumbnailPaintItem, AssetThumbnailGridMetrics,
    BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID,
};
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetViewMode, AssetWorkspaceSnapshot};

const THUMBNAIL_NAME_MAX_WIDTH: f32 = 96.0;
const THUMBNAIL_NAME_PRIMARY_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_BODY_SIZE;
const THUMBNAIL_NAME_CONTINUATION_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
const THUMBNAIL_NAME_PRIMARY_FONT_WEIGHT: i32 = 500;
const THUMBNAIL_NAME_CONTINUATION_FONT_WEIGHT: i32 = 400;
const THUMBNAIL_TYPE_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
const THUMBNAIL_META_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
const THUMBNAIL_CARD_SURFACE: &str = "asset-thumbnail-card";
const THUMBNAIL_NAME_AREA_SURFACE: &str = "asset-thumbnail-name-area";
const THUMBNAIL_NAME_AREA_TEXT_ROLE: &str = "asset-thumbnail-name-area-text";
const THUMBNAIL_NODES_PER_ITEM: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ThumbnailNodeKind {
    Card,
    Visual,
    InfoBand,
    SelectionMarker,
    Name,
    NameContinuation,
    TypeBadge,
    Type,
    Meta,
}

fn thumbnail_corner_radius() -> f32 {
    EditorControlTokens::workbench_dense().small_radius
}

pub(super) fn append_asset_browser_thumbnail_slots(
    nodes: &mut Vec<ViewTemplateNodeData>,
    snapshot: &AssetWorkspaceSnapshot,
    materialized_item_count: usize,
) {
    if snapshot.view_mode != AssetViewMode::Thumbnail {
        return;
    }

    let additional_nodes =
        thumbnail_node_capacity(snapshot.visible_assets.len(), materialized_item_count);
    nodes.reserve(additional_nodes);
    nodes.push(thumbnail_grid_panel());
    for (index, asset) in snapshot
        .visible_assets
        .iter()
        .take(materialized_item_count)
        .enumerate()
    {
        let selected =
            asset.selected || snapshot.selected_asset_uuid.as_deref() == Some(asset.uuid.as_str());
        nodes.push(thumbnail_card_node(index, selected));
        nodes.push(thumbnail_visual_node(index, selected, asset));
        nodes.push(thumbnail_info_band_node(index, selected));
        nodes.push(thumbnail_selection_marker_node(index));
        let file_like_name =
            is_file_like_thumbnail_name(asset.display_name.as_str(), asset.extension.as_str());
        let (name, name_continuation) = thumbnail_display_name_lines(asset);
        nodes.push(thumbnail_name_node(
            index,
            name,
            file_like_name.then(|| asset.display_name.clone()),
            selected,
        ));
        nodes.push(thumbnail_name_continuation_node(
            index,
            name_continuation,
            selected,
        ));
        nodes.push(thumbnail_type_badge_node(index));
        nodes.push(thumbnail_type_node(index, asset, selected));
        nodes.push(thumbnail_meta_node(index, asset, selected));
    }
}

fn thumbnail_node_capacity(visible_item_count: usize, materialized_item_count: usize) -> usize {
    1usize.saturating_add(
        visible_item_count
            .min(materialized_item_count)
            .saturating_mul(THUMBNAIL_NODES_PER_ITEM),
    )
}

pub(super) fn asset_browser_thumbnail_paint_item(
    asset: &AssetItemSnapshot,
) -> AssetBrowserPaintItem {
    let file_like_name =
        is_file_like_thumbnail_name(asset.display_name.as_str(), asset.extension.as_str());
    let (name, name_continuation) = thumbnail_display_name_lines(asset);
    AssetBrowserPaintItem::Thumbnail(AssetBrowserThumbnailPaintItem {
        name,
        source_file_name: file_like_name
            .then(|| asset.display_name.clone())
            .unwrap_or_default(),
        file_extension: file_like_name
            .then(|| asset.extension.clone())
            .unwrap_or_default(),
        name_continuation,
        type_label: asset.asset_type.badge.clone(),
        type_label_width: measure_runtime_text_width(
            asset.asset_type.badge.as_str(),
            THUMBNAIL_TYPE_FONT_SIZE,
        ),
        state_label: asset_state_label(asset).to_string(),
        visual_variant: asset.asset_type.icon_name.clone(),
        preview_artifact_path: asset.preview_artifact_path.clone(),
    })
}

pub(super) fn trim_asset_browser_thumbnail_slots(
    nodes: &mut Vec<ViewTemplateNodeData>,
    logical_item_count: usize,
    overscan_rows: usize,
) {
    let Some(grid) = nodes
        .iter()
        .find(|node| node.control_id == BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID)
    else {
        return;
    };
    let materialized_item_count =
        AssetThumbnailGridMetrics::new(grid.frame.width, logical_item_count)
            .materialized_item_budget(grid.frame.height, overscan_rows);
    nodes.retain(|node| {
        thumbnail_node_identity(node.control_id.as_str())
            .map(|(_, index)| index < materialized_item_count)
            .unwrap_or(true)
    });
}

#[cfg(test)]
#[path = "thumbnail_nodes/tests/capacity_tests.rs"]
mod capacity_tests;

#[cfg(test)]
fn append_asset_browser_thumbnail_nodes(
    nodes: &mut Vec<ViewTemplateNodeData>,
    snapshot: &AssetWorkspaceSnapshot,
) {
    append_asset_browser_thumbnail_slots(nodes, snapshot, snapshot.visible_assets.len());
}

fn thumbnail_grid_panel() -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: "AssetBrowserThumbGridPanel".into(),
        control_id: "AssetBrowserThumbGridPanel".into(),
        role: "Panel".into(),
        surface_variant: "frame_only".into(),
        frame: ViewTemplateFrameData::default(),
        ..ViewTemplateNodeData::default()
    }
}

fn thumbnail_card_node(index: usize, selected: bool) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: thumbnail_node_id("Card", index).into(),
        control_id: thumbnail_control_id("Card", index).into(),
        role: "Panel".into(),
        surface_variant: THUMBNAIL_CARD_SURFACE.into(),
        corner_radius: thumbnail_corner_radius(),
        border_width: if selected { 1.0 } else { 0.0 },
        selected,
        frame: ViewTemplateFrameData::default(),
        ..ViewTemplateNodeData::default()
    }
}

fn thumbnail_info_band_node(index: usize, selected: bool) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: thumbnail_node_id("InfoBand", index).into(),
        control_id: thumbnail_control_id("InfoBand", index).into(),
        role: "Panel".into(),
        surface_variant: THUMBNAIL_NAME_AREA_SURFACE.into(),
        corner_radius: thumbnail_corner_radius(),
        selected,
        frame: ViewTemplateFrameData::default(),
        ..ViewTemplateNodeData::default()
    }
}

fn thumbnail_selection_marker_node(index: usize) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: thumbnail_node_id("SelectionMarker", index).into(),
        control_id: thumbnail_control_id("SelectionMarker", index).into(),
        role: "Panel".into(),
        surface_variant: "accent".into(),
        frame: ViewTemplateFrameData::default(),
        ..ViewTemplateNodeData::default()
    }
}

fn thumbnail_type_badge_node(index: usize) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: thumbnail_node_id("TypeBadge", index).into(),
        control_id: thumbnail_control_id("TypeBadge", index).into(),
        role: "Panel".into(),
        surface_variant: "asset-type-badge".into(),
        corner_radius: thumbnail_corner_radius(),
        frame: ViewTemplateFrameData::default(),
        ..ViewTemplateNodeData::default()
    }
}

fn thumbnail_visual_node(
    index: usize,
    selected: bool,
    asset: &AssetItemSnapshot,
) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: thumbnail_node_id("Visual", index).into(),
        control_id: thumbnail_control_id("Visual", index).into(),
        role: "Panel".into(),
        component_role: "asset-thumbnail-visual".into(),
        component_variant: asset.asset_type.icon_name.clone().into(),
        surface_variant: if selected {
            "asset-preview-visual".into()
        } else {
            "asset-placeholder-visual".into()
        },
        media_source: asset.preview_artifact_path.clone().into(),
        has_preview_image: !asset.preview_artifact_path.trim().is_empty(),
        preview_image: Default::default(),
        corner_radius: thumbnail_corner_radius(),
        frame: ViewTemplateFrameData::default(),
        ..ViewTemplateNodeData::default()
    }
}

fn thumbnail_name_node(
    index: usize,
    text: String,
    source_file_name: Option<String>,
    selected: bool,
) -> ViewTemplateNodeData {
    let mut node = thumbnail_label_node(
        thumbnail_node_id("Name", index),
        thumbnail_control_id("Name", index),
        text,
        THUMBNAIL_NAME_PRIMARY_FONT_SIZE,
        THUMBNAIL_NAME_PRIMARY_FONT_WEIGHT,
        "",
        selected,
    );
    node.value_text = source_file_name.unwrap_or_default().into();
    node
}

fn thumbnail_name_continuation_node(
    index: usize,
    text: String,
    selected: bool,
) -> ViewTemplateNodeData {
    thumbnail_label_node(
        thumbnail_node_id("NameContinuation", index),
        thumbnail_control_id("NameContinuation", index),
        text,
        THUMBNAIL_NAME_CONTINUATION_FONT_SIZE,
        THUMBNAIL_NAME_CONTINUATION_FONT_WEIGHT,
        "muted",
        selected,
    )
}

fn thumbnail_type_node(
    index: usize,
    asset: &AssetItemSnapshot,
    selected: bool,
) -> ViewTemplateNodeData {
    thumbnail_label_node(
        thumbnail_node_id("Type", index),
        thumbnail_control_id("Type", index),
        asset.asset_type.badge.clone(),
        THUMBNAIL_TYPE_FONT_SIZE,
        700,
        "accent",
        selected,
    )
}

fn thumbnail_meta_node(
    index: usize,
    asset: &AssetItemSnapshot,
    selected: bool,
) -> ViewTemplateNodeData {
    thumbnail_label_node(
        thumbnail_node_id("Meta", index),
        thumbnail_control_id("Meta", index),
        asset_state_label(asset).to_string(),
        THUMBNAIL_META_FONT_SIZE,
        400,
        "muted",
        selected,
    )
}

fn thumbnail_label_node(
    node_id: String,
    control_id: String,
    text: String,
    font_size: f32,
    font_weight: i32,
    text_tone: &str,
    selected: bool,
) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: node_id.into(),
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        component_role: THUMBNAIL_NAME_AREA_TEXT_ROLE.into(),
        text_tone: text_tone.into(),
        selected,
        overflow: "elide".into(),
        font_size,
        font_weight,
        options: model_rc(Vec::<SharedString>::new()),
        frame: ViewTemplateFrameData::default(),
        ..ViewTemplateNodeData::default()
    }
}

fn thumbnail_display_name_lines(asset: &AssetItemSnapshot) -> (String, String) {
    let name = asset.display_name.trim();
    if is_file_like_thumbnail_name(name, asset.extension.as_str()) {
        return (name.to_string(), String::new());
    }

    asset_display_name_lines(name)
}

fn is_file_like_thumbnail_name(display_name: &str, extension: &str) -> bool {
    let extension = extension.trim().trim_start_matches('.');
    if extension.is_empty() {
        return false;
    }

    display_name
        .rsplit_once('.')
        .is_some_and(|(_, suffix)| suffix.eq_ignore_ascii_case(extension))
}

pub(super) fn compact_thumbnail_file_name_to_width(display_name: &str, max_width: f32) -> String {
    let Some((_, extension)) = display_name.rsplit_once('.') else {
        return display_name.to_string();
    };
    compact_thumbnail_name_to_width(
        display_name,
        extension,
        max_width,
        THUMBNAIL_NAME_PRIMARY_FONT_SIZE,
    )
}

pub(super) fn asset_display_name_lines(display_name: &str) -> (String, String) {
    split_display_name_lines(
        display_name,
        RuntimeNameLineSplit {
            max_width: THUMBNAIL_NAME_MAX_WIDTH,
            primary_font_size: THUMBNAIL_NAME_PRIMARY_FONT_SIZE,
            continuation_font_size: THUMBNAIL_NAME_CONTINUATION_FONT_SIZE,
        },
    )
}

fn thumbnail_node_id(kind: &str, index: usize) -> String {
    format!(
        "asset_browser.thumbnail.{}.{}",
        kind.to_ascii_lowercase(),
        index + 1
    )
}

pub(super) fn thumbnail_control_id(kind: &str, index: usize) -> String {
    format!("AssetBrowserThumb{kind}{:02}", index + 1)
}

pub(super) fn thumbnail_node_identity(control_id: &str) -> Option<(ThumbnailNodeKind, usize)> {
    let suffix = control_id.strip_prefix("AssetBrowserThumb")?;
    for (kind_name, kind) in [
        ("NameContinuation", ThumbnailNodeKind::NameContinuation),
        ("SelectionMarker", ThumbnailNodeKind::SelectionMarker),
        ("InfoBand", ThumbnailNodeKind::InfoBand),
        ("TypeBadge", ThumbnailNodeKind::TypeBadge),
        ("Visual", ThumbnailNodeKind::Visual),
        ("Card", ThumbnailNodeKind::Card),
        ("Name", ThumbnailNodeKind::Name),
        ("Type", ThumbnailNodeKind::Type),
        ("Meta", ThumbnailNodeKind::Meta),
    ] {
        let Some(number) = suffix.strip_prefix(kind_name) else {
            continue;
        };
        return number
            .parse::<usize>()
            .ok()?
            .checked_sub(1)
            .map(|index| (kind, index));
    }
    None
}

#[cfg(test)]
#[path = "tests/thumbnail_nodes.rs"]
mod tests;

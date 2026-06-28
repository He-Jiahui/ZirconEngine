use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetViewMode, AssetWorkspaceSnapshot};
use zircon_runtime_interface::resource::ResourceKind;

use super::{asset_state_label, compact_resource_kind_label};

const THUMBNAIL_GRID_MAX_ITEMS: usize = 8;
const THUMBNAIL_NAME_SINGLE_LINE_LIMIT: usize = 20;
const THUMBNAIL_NAME_MIN_LINE_CHARS: usize = 6;
const THUMBNAIL_NAME_TARGET_MIN_CHARS: usize = 12;
const THUMBNAIL_NAME_TARGET_MAX_CHARS: usize = 18;
const THUMBNAIL_NAME_PRIMARY_FONT_SIZE: f32 = 9.0;
const THUMBNAIL_NAME_CONTINUATION_FONT_SIZE: f32 = 8.0;
const THUMBNAIL_NAME_PRIMARY_FONT_WEIGHT: i32 = 500;
const THUMBNAIL_NAME_CONTINUATION_FONT_WEIGHT: i32 = 400;
const THUMBNAIL_META_FONT_SIZE: f32 = 8.0;
const THUMBNAIL_CARD_SURFACE: &str = "asset-thumbnail-card";
const THUMBNAIL_NAME_AREA_SURFACE: &str = "asset-thumbnail-name-area";

pub(super) fn append_asset_browser_thumbnail_nodes(
    nodes: &mut Vec<ViewTemplateNodeData>,
    snapshot: &AssetWorkspaceSnapshot,
) {
    if snapshot.view_mode != AssetViewMode::Thumbnail {
        return;
    }

    nodes.push(thumbnail_grid_panel());
    for (index, asset) in snapshot
        .visible_assets
        .iter()
        .take(THUMBNAIL_GRID_MAX_ITEMS)
        .enumerate()
    {
        let selected =
            asset.selected || snapshot.selected_asset_uuid.as_deref() == Some(asset.uuid.as_str());
        nodes.push(thumbnail_card_node(index, selected));
        nodes.push(thumbnail_visual_node(index, selected, asset.kind));
        nodes.push(thumbnail_info_band_node(index, selected));
        if selected {
            nodes.push(thumbnail_selection_marker_node(index));
        }
        let (name, name_continuation) = asset_display_name_lines(asset.display_name.as_str());
        nodes.push(thumbnail_name_node(index, name));
        nodes.push(thumbnail_name_continuation_node(index, name_continuation));
        nodes.push(thumbnail_type_badge_node(index));
        nodes.push(thumbnail_type_node(index, asset));
        nodes.push(thumbnail_meta_node(index, asset));
    }
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
        corner_radius: 4.0,
        border_width: 0.0,
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
        corner_radius: 4.0,
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
        corner_radius: 3.0,
        frame: ViewTemplateFrameData::default(),
        ..ViewTemplateNodeData::default()
    }
}

fn thumbnail_visual_node(index: usize, selected: bool, kind: ResourceKind) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: thumbnail_node_id("Visual", index).into(),
        control_id: thumbnail_control_id("Visual", index).into(),
        role: "Panel".into(),
        component_role: "asset-thumbnail-visual".into(),
        component_variant: asset_thumbnail_icon_name(kind).into(),
        surface_variant: if selected {
            "asset-preview-visual".into()
        } else {
            "asset-placeholder-visual".into()
        },
        corner_radius: 4.0,
        frame: ViewTemplateFrameData::default(),
        ..ViewTemplateNodeData::default()
    }
}

pub(super) fn asset_thumbnail_icon_name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Texture => "asset-texture",
        ResourceKind::Material | ResourceKind::MaterialGraph | ResourceKind::PhysicsMaterial => {
            "asset-material"
        }
        ResourceKind::Scene => "asset-scene",
        ResourceKind::Model
        | ResourceKind::Mesh
        | ResourceKind::NavMesh
        | ResourceKind::Terrain => "asset-mesh",
        ResourceKind::Shader => "asset-shader",
        ResourceKind::Sound => "asset-audio",
        ResourceKind::Font => "asset-font",
        ResourceKind::Prefab => "asset-prefab",
        ResourceKind::AnimationSkeleton
        | ResourceKind::AnimationClip
        | ResourceKind::AnimationSequence
        | ResourceKind::AnimationGraph
        | ResourceKind::AnimationStateMachine => "asset-animation-clip",
        ResourceKind::TileSet | ResourceKind::TileMap => "asset-tilemap",
        ResourceKind::Data | ResourceKind::NavigationSettings | ResourceKind::TerrainLayerStack => {
            "asset-script"
        }
        ResourceKind::UiLayout => "asset-ui-layout",
        ResourceKind::UiWidget => "asset-ui-widget",
        ResourceKind::UiStyle => "asset-ui-style",
    }
}

fn thumbnail_name_node(index: usize, text: String) -> ViewTemplateNodeData {
    thumbnail_label_node(
        thumbnail_node_id("Name", index),
        thumbnail_control_id("Name", index),
        text,
        THUMBNAIL_NAME_PRIMARY_FONT_SIZE,
        THUMBNAIL_NAME_PRIMARY_FONT_WEIGHT,
        "",
    )
}

fn thumbnail_name_continuation_node(index: usize, text: String) -> ViewTemplateNodeData {
    thumbnail_label_node(
        thumbnail_node_id("NameContinuation", index),
        thumbnail_control_id("NameContinuation", index),
        text,
        THUMBNAIL_NAME_CONTINUATION_FONT_SIZE,
        THUMBNAIL_NAME_CONTINUATION_FONT_WEIGHT,
        "muted",
    )
}

fn thumbnail_type_node(index: usize, asset: &AssetItemSnapshot) -> ViewTemplateNodeData {
    thumbnail_label_node(
        thumbnail_node_id("Type", index),
        thumbnail_control_id("Type", index),
        compact_resource_kind_label(asset.kind).to_ascii_uppercase(),
        8.0,
        700,
        "accent",
    )
}

fn thumbnail_meta_node(index: usize, asset: &AssetItemSnapshot) -> ViewTemplateNodeData {
    thumbnail_label_node(
        thumbnail_node_id("Meta", index),
        thumbnail_control_id("Meta", index),
        asset_state_label(asset).to_string(),
        THUMBNAIL_META_FONT_SIZE,
        400,
        "muted",
    )
}

fn thumbnail_label_node(
    node_id: String,
    control_id: String,
    text: String,
    font_size: f32,
    font_weight: i32,
    text_tone: &str,
) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: node_id.into(),
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        text_tone: text_tone.into(),
        overflow: "elide".into(),
        font_size,
        font_weight,
        options: model_rc(Vec::<SharedString>::new()),
        frame: ViewTemplateFrameData::default(),
        ..ViewTemplateNodeData::default()
    }
}

pub(super) fn asset_display_name_lines(display_name: &str) -> (String, String) {
    let name = display_name.trim();
    let char_count = name.chars().count();
    if char_count <= THUMBNAIL_NAME_SINGLE_LINE_LIMIT {
        return (name.to_string(), String::new());
    }

    let split_byte = thumbnail_name_split_byte(name, char_count);
    let (first, second) = name.split_at(split_byte);
    let first = thumbnail_name_line_text(first);
    let second = thumbnail_name_line_text(second);
    if first.is_empty() || second.is_empty() {
        let fallback_byte = byte_index_at_char(name, thumbnail_name_target(char_count));
        let (fallback_first, fallback_second) = name.split_at(fallback_byte);
        return (
            fallback_first.trim().to_string(),
            fallback_second.trim().to_string(),
        );
    }

    (first, second)
}

fn thumbnail_name_split_byte(name: &str, char_count: usize) -> usize {
    let target = thumbnail_name_target(char_count);
    let mut candidates = Vec::new();
    collect_separator_breaks(name, char_count, &mut candidates);
    collect_camel_case_breaks(name, char_count, &mut candidates);
    let split_char = candidates
        .into_iter()
        .min_by_key(|candidate| (candidate.abs_diff(target), *candidate > target))
        .unwrap_or(target);
    byte_index_at_char(name, split_char)
}

fn thumbnail_name_target(char_count: usize) -> usize {
    (char_count / 2).clamp(
        THUMBNAIL_NAME_TARGET_MIN_CHARS,
        THUMBNAIL_NAME_TARGET_MAX_CHARS,
    )
}

fn collect_separator_breaks(name: &str, char_count: usize, candidates: &mut Vec<usize>) {
    for (index, ch) in name.chars().enumerate() {
        if is_thumbnail_name_separator(ch) && is_valid_thumbnail_name_break(index, char_count) {
            candidates.push(index);
        }
    }
}

fn collect_camel_case_breaks(name: &str, char_count: usize, candidates: &mut Vec<usize>) {
    let mut previous: Option<char> = None;
    for (index, ch) in name.chars().enumerate() {
        if let Some(previous) = previous {
            let is_boundary = ch.is_ascii_uppercase()
                && (previous.is_ascii_lowercase() || previous.is_ascii_digit());
            if is_boundary && is_valid_thumbnail_name_break(index, char_count) {
                candidates.push(index);
            }
        }
        previous = Some(ch);
    }
}

fn is_valid_thumbnail_name_break(index: usize, char_count: usize) -> bool {
    index >= THUMBNAIL_NAME_MIN_LINE_CHARS
        && char_count.saturating_sub(index) >= THUMBNAIL_NAME_MIN_LINE_CHARS
}

fn is_thumbnail_name_separator(ch: char) -> bool {
    matches!(ch, '_' | '-' | '.' | '/' | '\\')
}

fn thumbnail_name_line_text(text: &str) -> String {
    text.trim_matches(|ch: char| ch.is_whitespace() || is_thumbnail_name_separator(ch))
        .to_string()
}

fn byte_index_at_char(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(text.len())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;
    use zircon_runtime_interface::resource::ResourceKind;

    #[test]
    fn thumbnail_nodes_are_only_appended_for_thumbnail_view() {
        let mut nodes = Vec::new();
        append_asset_browser_thumbnail_nodes(&mut nodes, &AssetWorkspaceSnapshot::default());
        assert!(nodes.is_empty());
    }

    #[test]
    fn thumbnail_nodes_project_selected_asset_card_and_labels() {
        let mut snapshot = AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::Thumbnail,
            selected_asset_uuid: Some("asset-a".to_string()),
            visible_assets: vec![AssetItemSnapshot {
                uuid: "asset-a".to_string(),
                locator: "res://a".to_string(),
                display_name: "A_Texture.png".to_string(),
                file_name: "A_Texture.png".to_string(),
                extension: "png".to_string(),
                kind: ResourceKind::Texture,
                preview_artifact_path: String::new(),
                dirty: false,
                diagnostics: Vec::new(),
                selected: false,
                resource_state: None,
                resource_revision: Some(1),
            }],
            ..AssetWorkspaceSnapshot::default()
        };
        let mut nodes = Vec::new();
        append_asset_browser_thumbnail_nodes(&mut nodes, &snapshot);

        assert!(nodes
            .iter()
            .any(|node| node.control_id == "AssetBrowserThumbCard01"
                && node.selected
                && !node.focused
                && node.surface_variant == THUMBNAIL_CARD_SURFACE
                && node.border_width == 0.0));
        assert!(nodes
            .iter()
            .any(|node| node.control_id == "AssetBrowserThumbInfoBand01"
                && node.selected
                && node.surface_variant == THUMBNAIL_NAME_AREA_SURFACE
                && node.corner_radius == 4.0));
        assert!(nodes.iter().any(|node| {
            node.control_id == "AssetBrowserThumbSelectionMarker01"
                && node.surface_variant == "accent"
        }));
        assert!(nodes
            .iter()
            .any(|node| node.control_id == "AssetBrowserThumbVisual01"
                && node.surface_variant == "asset-preview-visual"
                && node.component_role == "asset-thumbnail-visual"
                && node.component_variant == "asset-texture"));
        assert!(nodes.iter().any(|node| {
            node.control_id == "AssetBrowserThumbName01"
                && node.role == "Label"
                && node.text == "A_Texture.png"
                && node.font_size == THUMBNAIL_NAME_PRIMARY_FONT_SIZE
                && node.font_weight == THUMBNAIL_NAME_PRIMARY_FONT_WEIGHT
        }));
        assert!(nodes.iter().any(|node| {
            node.control_id == "AssetBrowserThumbNameContinuation01"
                && node.role == "Label"
                && node.text.is_empty()
                && node.font_size == THUMBNAIL_NAME_CONTINUATION_FONT_SIZE
                && node.font_weight == THUMBNAIL_NAME_CONTINUATION_FONT_WEIGHT
                && node.text_tone == "muted"
        }));
        assert!(nodes.iter().any(|node| {
            node.control_id == "AssetBrowserThumbTypeBadge01"
                && node.role == "Panel"
                && node.surface_variant == "asset-type-badge"
                && node.corner_radius == 3.0
        }));
        assert!(nodes.iter().any(|node| {
            node.control_id == "AssetBrowserThumbType01"
                && node.role == "Label"
                && node.text == "TEX"
                && node.text_tone == "accent"
                && node.font_size == 8.0
                && node.font_weight == 700
        }));
        assert!(nodes
            .iter()
            .any(|node| node.control_id == "AssetBrowserThumbMeta01"
                && node.text == "Ready"
                && node.text_tone == "muted"));

        snapshot.view_mode = AssetViewMode::List;
        nodes.clear();
        append_asset_browser_thumbnail_nodes(&mut nodes, &snapshot);
        assert!(nodes.is_empty());
    }

    #[test]
    fn thumbnail_nodes_project_long_asset_names_as_two_tile_lines() {
        let snapshot = AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::Thumbnail,
            visible_assets: vec![AssetItemSnapshot {
                uuid: "asset-ui-layout".to_string(),
                locator: "res://ui/editor/workbench_host_window.zui".to_string(),
                display_name: "workbench_host_window.zui".to_string(),
                file_name: "workbench_host_window.zui".to_string(),
                extension: "zui".to_string(),
                kind: ResourceKind::UiLayout,
                preview_artifact_path: String::new(),
                dirty: false,
                diagnostics: Vec::new(),
                selected: true,
                resource_state: None,
                resource_revision: Some(42),
            }],
            ..AssetWorkspaceSnapshot::default()
        };
        let mut nodes = Vec::new();
        append_asset_browser_thumbnail_nodes(&mut nodes, &snapshot);

        let name = nodes
            .iter()
            .find(|node| node.control_id == "AssetBrowserThumbName01")
            .expect("missing thumbnail primary name");
        let continuation = nodes
            .iter()
            .find(|node| node.control_id == "AssetBrowserThumbNameContinuation01")
            .expect("missing thumbnail continuation name");

        assert_eq!(name.text.as_str(), "workbench_host");
        assert_eq!(continuation.text.as_str(), "window.zui");
        assert_eq!(name.overflow.as_str(), "elide");
        assert_eq!(continuation.overflow.as_str(), "elide");
        assert_eq!(name.font_size, THUMBNAIL_NAME_PRIMARY_FONT_SIZE);
        assert_eq!(
            continuation.font_size,
            THUMBNAIL_NAME_CONTINUATION_FONT_SIZE
        );
        assert_eq!(continuation.text_tone.as_str(), "muted");
    }

    #[test]
    fn thumbnail_nodes_project_type_specific_visual_icons() {
        let snapshot = AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::Thumbnail,
            visible_assets: vec![
                asset("asset-texture", ResourceKind::Texture),
                asset("asset-material", ResourceKind::Material),
                asset("asset-scene", ResourceKind::Scene),
                asset("asset-shader", ResourceKind::Shader),
                asset("asset-mesh", ResourceKind::Mesh),
                asset("asset-ui-layout", ResourceKind::UiLayout),
            ],
            ..AssetWorkspaceSnapshot::default()
        };
        let mut nodes = Vec::new();
        append_asset_browser_thumbnail_nodes(&mut nodes, &snapshot);

        for (index, expected_icon) in [
            "asset-texture",
            "asset-material",
            "asset-scene",
            "asset-shader",
            "asset-mesh",
            "asset-ui-layout",
        ]
        .iter()
        .enumerate()
        {
            let control_id = thumbnail_control_id("Visual", index);
            let visual = nodes
                .iter()
                .find(|node| node.control_id == control_id)
                .unwrap_or_else(|| panic!("missing thumbnail visual {control_id}"));
            assert_eq!(visual.component_role.as_str(), "asset-thumbnail-visual");
            assert_eq!(visual.component_variant.as_str(), *expected_icon);
            assert!(visual.icon_name.is_empty());
        }
    }

    fn asset(uuid: &str, kind: ResourceKind) -> AssetItemSnapshot {
        AssetItemSnapshot {
            uuid: uuid.to_string(),
            locator: format!("res://{uuid}"),
            display_name: format!("{uuid}.asset"),
            file_name: format!("{uuid}.asset"),
            extension: "asset".to_string(),
            kind,
            preview_artifact_path: String::new(),
            dirty: false,
            diagnostics: Vec::new(),
            selected: false,
            resource_state: None,
            resource_revision: Some(1),
        }
    }
}

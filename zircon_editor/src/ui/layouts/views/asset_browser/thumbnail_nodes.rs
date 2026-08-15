use super::labels::asset_state_label;
use super::name_compaction::{RuntimeFileNameCompaction, compact_file_like_display_name};
use super::name_lines::{RuntimeNameLineSplit, split_display_name_lines};
use zircon_runtime_interface::ui::design_tokens::{EditorControlTokens, EditorTypographyTokens};

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{
    ViewTemplateFrameData, ViewTemplateNodeData, load_preview_image_for_generation,
};
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetViewMode, AssetWorkspaceSnapshot};

const THUMBNAIL_NAME_MAX_WIDTH: f32 = 96.0;
const THUMBNAIL_FILE_NAME_MIN_PREFIX_CHARS: usize = 4;
const THUMBNAIL_FILE_NAME_MIN_TAIL_STEM_CHARS: usize = 3;
const THUMBNAIL_FILE_NAME_EXTENSION_TAIL_STEM_CHARS: usize = 4;
const THUMBNAIL_NAME_PRIMARY_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_BODY_SIZE;
const THUMBNAIL_NAME_CONTINUATION_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
const THUMBNAIL_NAME_PRIMARY_FONT_WEIGHT: i32 = 500;
const THUMBNAIL_NAME_CONTINUATION_FONT_WEIGHT: i32 = 400;
const THUMBNAIL_TYPE_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
const THUMBNAIL_META_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
const THUMBNAIL_CARD_SURFACE: &str = "asset-thumbnail-card";
const THUMBNAIL_NAME_AREA_SURFACE: &str = "asset-thumbnail-name-area";
const THUMBNAIL_NAME_AREA_TEXT_ROLE: &str = "asset-thumbnail-name-area-text";

fn thumbnail_corner_radius() -> f32 {
    EditorControlTokens::workbench_dense().small_radius
}

pub(super) fn append_asset_browser_thumbnail_nodes(
    nodes: &mut Vec<ViewTemplateNodeData>,
    snapshot: &AssetWorkspaceSnapshot,
) {
    if snapshot.view_mode != AssetViewMode::Thumbnail {
        return;
    }

    nodes.push(thumbnail_grid_panel());
    for (index, asset) in snapshot.visible_assets.iter().enumerate() {
        let selected =
            asset.selected || snapshot.selected_asset_uuid.as_deref() == Some(asset.uuid.as_str());
        nodes.push(thumbnail_card_node(index, selected));
        nodes.push(thumbnail_visual_node(
            index,
            selected,
            asset,
            snapshot.catalog_revision,
        ));
        nodes.push(thumbnail_info_band_node(index, selected));
        if selected {
            nodes.push(thumbnail_selection_marker_node(index));
        }
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
    workspace_generation: u64,
) -> ViewTemplateNodeData {
    let resource_generation = workspace_generation ^ asset.resource_revision.unwrap_or_default();
    let preview_image = load_preview_image_for_generation(
        asset.preview_artifact_path.as_str(),
        "",
        resource_generation,
    );
    let preview_size = preview_image.size();
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
        has_preview_image: preview_size.width > 0 && preview_size.height > 0,
        preview_image,
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
    compact_file_like_display_name(
        display_name,
        extension,
        RuntimeFileNameCompaction {
            max_width,
            font_size: THUMBNAIL_NAME_PRIMARY_FONT_SIZE,
            min_prefix_chars: THUMBNAIL_FILE_NAME_MIN_PREFIX_CHARS,
            min_tail_stem_chars: THUMBNAIL_FILE_NAME_MIN_TAIL_STEM_CHARS,
            preferred_tail_stem_chars: THUMBNAIL_FILE_NAME_EXTENSION_TAIL_STEM_CHARS,
        },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::measure_runtime_text_width;
    use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;
    use zircon_runtime_interface::resource::ResourceKind;

    #[test]
    fn thumbnail_nodes_are_only_appended_for_thumbnail_view() {
        let mut nodes = Vec::new();
        append_asset_browser_thumbnail_nodes(&mut nodes, &AssetWorkspaceSnapshot::default());
        assert!(nodes.is_empty());
    }

    #[test]
    fn thumbnail_node_typography_and_corner_radius_follow_workbench_tokens() {
        assert_eq!(
            THUMBNAIL_NAME_PRIMARY_FONT_SIZE,
            zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens::WORKBENCH_BODY_SIZE
        );
        assert_eq!(
            THUMBNAIL_NAME_CONTINUATION_FONT_SIZE,
            zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
        );
        assert_eq!(
            THUMBNAIL_TYPE_FONT_SIZE,
            zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
        );
        assert_eq!(
            THUMBNAIL_META_FONT_SIZE,
            zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
        );
        assert_eq!(
            thumbnail_corner_radius(),
            zircon_runtime_interface::ui::design_tokens::EditorControlTokens::workbench_dense()
                .small_radius
        );
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
                asset_type:
                    crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
                        ResourceKind::Texture,
                    ),
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

        assert!(
            nodes
                .iter()
                .any(|node| node.control_id == "AssetBrowserThumbCard01"
                    && node.selected
                    && !node.focused
                    && node.surface_variant == THUMBNAIL_CARD_SURFACE
                    && node.corner_radius == thumbnail_corner_radius()
                    && node.border_width == 1.0)
        );
        assert!(
            nodes
                .iter()
                .any(|node| node.control_id == "AssetBrowserThumbInfoBand01"
                    && node.selected
                    && node.surface_variant == THUMBNAIL_NAME_AREA_SURFACE
                    && node.corner_radius == thumbnail_corner_radius())
        );
        assert!(nodes.iter().any(|node| {
            node.control_id == "AssetBrowserThumbSelectionMarker01"
                && node.surface_variant == "accent"
        }));
        assert!(
            nodes
                .iter()
                .any(|node| node.control_id == "AssetBrowserThumbVisual01"
                    && node.surface_variant == "asset-preview-visual"
                    && node.component_role == "asset-thumbnail-visual"
                    && node.component_variant == "asset-texture")
        );
        assert!(nodes.iter().any(|node| {
            node.control_id == "AssetBrowserThumbName01"
                && node.role == "Label"
                && node.component_role == THUMBNAIL_NAME_AREA_TEXT_ROLE
                && node.selected
                && node.text == "A_Texture.png"
                && node.font_size == THUMBNAIL_NAME_PRIMARY_FONT_SIZE
                && node.font_weight == THUMBNAIL_NAME_PRIMARY_FONT_WEIGHT
        }));
        assert!(nodes.iter().any(|node| {
            node.control_id == "AssetBrowserThumbNameContinuation01"
                && node.role == "Label"
                && node.component_role == THUMBNAIL_NAME_AREA_TEXT_ROLE
                && node.selected
                && node.text.is_empty()
                && node.font_size == THUMBNAIL_NAME_CONTINUATION_FONT_SIZE
                && node.font_weight == THUMBNAIL_NAME_CONTINUATION_FONT_WEIGHT
                && node.text_tone == "muted"
        }));
        assert!(nodes.iter().any(|node| {
            node.control_id == "AssetBrowserThumbTypeBadge01"
                && node.role == "Panel"
                && node.surface_variant == "asset-type-badge"
                && node.corner_radius == thumbnail_corner_radius()
        }));
        assert!(nodes.iter().any(|node| {
            node.control_id == "AssetBrowserThumbType01"
                && node.role == "Label"
                && node.component_role == THUMBNAIL_NAME_AREA_TEXT_ROLE
                && node.selected
                && node.text == "TEX"
                && node.text_tone == "accent"
                && node.font_size == THUMBNAIL_TYPE_FONT_SIZE
                && node.font_weight == 700
        }));
        assert!(
            nodes
                .iter()
                .any(|node| node.control_id == "AssetBrowserThumbMeta01"
                    && node.component_role == THUMBNAIL_NAME_AREA_TEXT_ROLE
                    && node.selected
                    && node.text == "Ready"
                    && node.text_tone == "muted")
        );

        snapshot.view_mode = AssetViewMode::List;
        nodes.clear();
        append_asset_browser_thumbnail_nodes(&mut nodes, &snapshot);
        assert!(nodes.is_empty());
    }

    #[test]
    fn thumbnail_nodes_keep_file_like_asset_names_on_one_extension_preserving_tile_line() {
        let snapshot = AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::Thumbnail,
            visible_assets: vec![AssetItemSnapshot {
                uuid: "asset-ui-layout".to_string(),
                locator: "res://ui/editor/workbench_host_window.zui".to_string(),
                display_name: "workbench_host_window.zui".to_string(),
                file_name: "workbench_host_window.zui".to_string(),
                extension: "zui".to_string(),
                kind: ResourceKind::UiLayout,
                asset_type:
                    crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
                        ResourceKind::UiLayout,
                    ),
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

        assert_eq!(name.text.as_str(), "workbench_host_window.zui");
        assert_eq!(name.value_text.as_str(), "workbench_host_window.zui");
        assert!(continuation.text.is_empty());
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
    fn thumbnail_nodes_preserve_long_file_extension_when_eliding_title() {
        let snapshot = AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::Thumbnail,
            visible_assets: vec![AssetItemSnapshot {
                uuid: "asset-scene-preview".to_string(),
                locator: "res://scene/editor_preview.zscene".to_string(),
                display_name: "editor_preview.zscene".to_string(),
                file_name: "editor_preview.zscene".to_string(),
                extension: "zscene".to_string(),
                kind: ResourceKind::Scene,
                asset_type:
                    crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
                        ResourceKind::Scene,
                    ),
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

        assert_eq!(name.text.as_str(), "editor_preview.zscene");
        assert_eq!(name.value_text.as_str(), "editor_preview.zscene");
        assert!(continuation.text.is_empty());
    }

    #[test]
    fn thumbnail_file_like_title_uses_runtime_width_not_character_count() {
        let narrow = format!("{}.zui", "i".repeat(24));
        let wide = format!("{}.zui", "W".repeat(24));
        assert_eq!(narrow.chars().count(), wide.chars().count());

        assert_eq!(
            compact_thumbnail_file_name_to_width(&narrow, THUMBNAIL_NAME_MAX_WIDTH),
            narrow
        );
        let compact_wide = compact_thumbnail_file_name_to_width(&wide, THUMBNAIL_NAME_MAX_WIDTH);

        assert_ne!(compact_wide, wide);
        assert!(compact_wide.ends_with(".zui"));
        assert!(
            measure_runtime_text_width(&compact_wide, THUMBNAIL_NAME_PRIMARY_FONT_SIZE)
                <= THUMBNAIL_NAME_MAX_WIDTH + 0.01,
            "thumbnail file-like title should fit measured width: {compact_wide}"
        );
    }

    #[test]
    fn thumbnail_nodes_project_non_file_long_asset_names_as_two_tile_lines() {
        let snapshot = AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::Thumbnail,
            visible_assets: vec![AssetItemSnapshot {
                uuid: "asset-navigation-profile".to_string(),
                locator: "res://data/NavigationSettingsRuntimeProfile".to_string(),
                display_name: "NavigationSettingsRuntimeProfile".to_string(),
                file_name: "NavigationSettingsRuntimeProfile".to_string(),
                extension: String::new(),
                kind: ResourceKind::Data,
                asset_type:
                    crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
                        ResourceKind::Data,
                    ),
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

        assert_eq!(name.text.as_str(), "NavigationSettings");
        assert_eq!(continuation.text.as_str(), "RuntimeProfile");
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

    #[test]
    fn thumbnail_nodes_project_every_catalog_asset_without_a_fixed_item_cap() {
        let snapshot = AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::Thumbnail,
            visible_assets: (0..12)
                .map(|index| asset(&format!("asset-{index:02}"), ResourceKind::Texture))
                .collect(),
            ..AssetWorkspaceSnapshot::default()
        };
        let mut nodes = Vec::new();

        append_asset_browser_thumbnail_nodes(&mut nodes, &snapshot);

        assert!(
            nodes
                .iter()
                .any(|node| node.control_id == "AssetBrowserThumbCard12")
        );
        assert_eq!(
            nodes
                .iter()
                .filter(|node| node.control_id.starts_with("AssetBrowserThumbCard"))
                .count(),
            12
        );
    }

    #[test]
    fn thumbnail_nodes_project_preview_artifact_into_visual_node() {
        let preview_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/ui/editor/showcase_checker.svg")
            .to_string_lossy()
            .into_owned();
        let snapshot = AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::Thumbnail,
            visible_assets: vec![AssetItemSnapshot {
                uuid: "asset-preview-texture".to_string(),
                locator: "res://textures/preview".to_string(),
                display_name: "preview.texture".to_string(),
                file_name: "preview.texture".to_string(),
                extension: "texture".to_string(),
                kind: ResourceKind::Texture,
                asset_type:
                    crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
                        ResourceKind::Texture,
                    ),
                preview_artifact_path: preview_path.clone(),
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

        let visual = nodes
            .iter()
            .find(|node| node.control_id == "AssetBrowserThumbVisual01")
            .expect("thumbnail visual should exist");
        let preview_size = visual.preview_image.size();
        assert_eq!(visual.media_source.as_str(), preview_path.as_str());
        assert!(visual.has_preview_image);
        assert!(preview_size.width > 0 && preview_size.height > 0);
    }

    fn asset(uuid: &str, kind: ResourceKind) -> AssetItemSnapshot {
        AssetItemSnapshot {
            uuid: uuid.to_string(),
            locator: format!("res://{uuid}"),
            display_name: format!("{uuid}.asset"),
            file_name: format!("{uuid}.asset"),
            extension: "asset".to_string(),
            kind,
            asset_type:
                crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
                    kind,
                ),
            preview_artifact_path: String::new(),
            dirty: false,
            diagnostics: Vec::new(),
            selected: false,
            resource_state: None,
            resource_revision: Some(1),
        }
    }
}

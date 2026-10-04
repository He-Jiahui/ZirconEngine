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
        }]
        .into(),
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
            && node.corner_radius == thumbnail_corner_radius()
            && node.border_width == 1.0));
    assert!(nodes
        .iter()
        .any(|node| node.control_id == "AssetBrowserThumbInfoBand01"
            && node.selected
            && node.surface_variant == THUMBNAIL_NAME_AREA_SURFACE
            && node.corner_radius == thumbnail_corner_radius()));
    assert!(nodes.iter().any(|node| {
        node.control_id == "AssetBrowserThumbSelectionMarker01" && node.surface_variant == "accent"
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
    assert!(nodes
        .iter()
        .any(|node| node.control_id == "AssetBrowserThumbMeta01"
            && node.component_role == THUMBNAIL_NAME_AREA_TEXT_ROLE
            && node.selected
            && node.text == "Ready"
            && node.text_tone == "muted"));

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
        }]
        .into(),
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
        }]
        .into(),
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
        }]
        .into(),
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
        ]
        .into(),
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

    assert!(nodes
        .iter()
        .any(|node| node.control_id == "AssetBrowserThumbCard12"));
    assert_eq!(
        nodes
            .iter()
            .filter(|node| node.control_id.starts_with("AssetBrowserThumbCard"))
            .count(),
        12
    );
}

#[test]
fn thumbnail_nodes_defer_preview_artifact_pixels_to_visible_paint() {
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
        }]
        .into(),
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
    assert_eq!((preview_size.width, preview_size.height), (0, 0));
}

#[test]
fn thumbnail_node_identity_decodes_each_layout_part_without_allocating_an_index() {
    for (kind_name, expected_kind) in [
        ("Card", ThumbnailNodeKind::Card),
        ("Visual", ThumbnailNodeKind::Visual),
        ("InfoBand", ThumbnailNodeKind::InfoBand),
        ("SelectionMarker", ThumbnailNodeKind::SelectionMarker),
        ("Name", ThumbnailNodeKind::Name),
        ("NameContinuation", ThumbnailNodeKind::NameContinuation),
        ("TypeBadge", ThumbnailNodeKind::TypeBadge),
        ("Type", ThumbnailNodeKind::Type),
        ("Meta", ThumbnailNodeKind::Meta),
    ] {
        assert_eq!(
            thumbnail_node_identity(&thumbnail_control_id(kind_name, 41)),
            Some((expected_kind, 41))
        );
    }
    assert_eq!(thumbnail_node_identity("AssetBrowserThumbCard00"), None);
    assert_eq!(thumbnail_node_identity("AssetBrowserThumbUnknown42"), None);
}

fn asset(uuid: &str, kind: ResourceKind) -> AssetItemSnapshot {
    AssetItemSnapshot {
        uuid: uuid.to_string(),
        locator: format!("res://{uuid}"),
        display_name: format!("{uuid}.asset"),
        file_name: format!("{uuid}.asset"),
        extension: "asset".to_string(),
        kind,
        asset_type: crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
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

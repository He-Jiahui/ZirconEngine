use super::*;
use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetWorkspaceSnapshot};
use zircon_runtime_interface::resource::ResourceKind;

#[test]
fn summary_nodes_split_selected_asset_meta_into_badge_state_and_revision() {
    let snapshot = AssetWorkspaceSnapshot {
        selected_asset_uuid: Some("asset-a".to_string()),
        visible_assets: vec![AssetItemSnapshot {
            uuid: "asset-a".to_string(),
            locator: "res://a".to_string(),
            display_name: "A.zui".to_string(),
            file_name: "A.zui".to_string(),
            extension: "zui".to_string(),
            kind: ResourceKind::UiLayout,
            asset_type:
                crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
                    ResourceKind::UiLayout,
                ),
            preview_artifact_path: String::new(),
            dirty: false,
            diagnostics: Vec::new(),
            selected: false,
            resource_state: None,
            resource_revision: Some(42),
        }]
        .into(),
        ..AssetWorkspaceSnapshot::default()
    };
    let mut nodes = Vec::new();

    append_asset_browser_summary_nodes(&mut nodes, &snapshot);

    assert!(nodes.iter().any(|node| {
        node.control_id == SUMMARY_TYPE_BADGE_CONTROL_ID
            && node.role == "Panel"
            && node.surface_variant == "asset-type-badge"
            && node.corner_radius == summary_badge_corner_radius()
    }));
    assert!(nodes.iter().any(|node| {
        node.control_id == SUMMARY_TYPE_CONTROL_ID
            && node.text == "UI Layout"
            && node.text_tone == "accent"
            && node.font_size == SUMMARY_META_FONT_SIZE
            && node.font_weight == 700
    }));
    assert!(nodes.iter().any(|node| {
        node.control_id == SUMMARY_STATE_CONTROL_ID
            && node.text == "Ready"
            && node.text_tone == "muted"
    }));
    assert!(nodes
        .iter()
        .any(|node| { node.control_id == SUMMARY_REVISION_CONTROL_ID && node.text == "rev 42" }));
}

#[test]
fn summary_nodes_split_selected_long_name_into_primary_and_continuation_labels() {
    let snapshot = AssetWorkspaceSnapshot {
        selected_asset_uuid: Some("asset-a".to_string()),
        visible_assets: vec![AssetItemSnapshot {
            uuid: "asset-a".to_string(),
            locator: "res://a".to_string(),
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
            selected: false,
            resource_state: None,
            resource_revision: Some(42),
        }]
        .into(),
        ..AssetWorkspaceSnapshot::default()
    };
    let mut nodes = vec![ViewTemplateNodeData {
        node_id: "asset_browser.content_preview.name".into(),
        control_id: "AssetBrowserContentPreviewName".into(),
        role: "Label".into(),
        text: "NavigationSettingsRuntimeProfile".into(),
        frame: ViewTemplateFrameData::default(),
        ..ViewTemplateNodeData::default()
    }];

    append_asset_browser_summary_nodes(&mut nodes, &snapshot);

    let name = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserContentPreviewName")
        .expect("summary name node should exist");
    let continuation = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserContentPreviewNameContinuation")
        .expect("summary continuation node should exist");
    let (expected_primary, expected_continuation) = split_display_name_lines(
        "NavigationSettingsRuntimeProfile",
        RuntimeNameLineSplit {
            max_width: SUMMARY_FILE_NAME_MAX_WIDTH,
            primary_font_size: SUMMARY_NAME_FONT_SIZE,
            continuation_font_size: SUMMARY_NAME_CONTINUATION_FONT_SIZE,
        },
    );
    assert_eq!(name.text.as_str(), expected_primary.as_str());
    assert_eq!(name.font_size, SUMMARY_NAME_FONT_SIZE);
    assert_eq!(name.font_weight, 600);
    assert_eq!(continuation.text.as_str(), expected_continuation.as_str());
    assert_eq!(continuation.role.as_str(), "Label");
    assert_eq!(continuation.font_size, SUMMARY_NAME_CONTINUATION_FONT_SIZE);
    assert_eq!(continuation.font_weight, 500);
    assert_eq!(continuation.text_tone.as_str(), "muted");
}

#[test]
fn summary_nodes_keep_width_fitting_short_wide_names_on_single_line() {
    let wide_name = "WWWWWWWWWWWW";
    assert!(
        measure_runtime_text_width(wide_name, SUMMARY_NAME_FONT_SIZE)
            <= SUMMARY_FILE_NAME_MAX_WIDTH + 0.01
    );
    let snapshot = AssetWorkspaceSnapshot {
        selected_asset_uuid: Some("asset-a".to_string()),
        visible_assets: vec![AssetItemSnapshot {
            uuid: "asset-a".to_string(),
            locator: "res://a".to_string(),
            display_name: wide_name.to_string(),
            file_name: wide_name.to_string(),
            extension: String::new(),
            kind: ResourceKind::Data,
            asset_type:
                crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
                    ResourceKind::Data,
                ),
            preview_artifact_path: String::new(),
            dirty: false,
            diagnostics: Vec::new(),
            selected: false,
            resource_state: None,
            resource_revision: Some(42),
        }]
        .into(),
        ..AssetWorkspaceSnapshot::default()
    };
    let mut nodes = vec![ViewTemplateNodeData {
        node_id: "asset_browser.content_preview.name".into(),
        control_id: "AssetBrowserContentPreviewName".into(),
        role: "Label".into(),
        frame: ViewTemplateFrameData::default(),
        ..ViewTemplateNodeData::default()
    }];

    append_asset_browser_summary_nodes(&mut nodes, &snapshot);

    let name = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserContentPreviewName")
        .expect("summary name node should exist");
    let continuation = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserContentPreviewNameContinuation")
        .expect("summary continuation node should exist");
    assert_eq!(name.text.as_str(), wide_name);
    assert!(continuation.text.is_empty());
}

#[test]
fn summary_nodes_keep_file_like_selected_names_on_single_line() {
    let snapshot = AssetWorkspaceSnapshot {
        selected_asset_uuid: Some("asset-a".to_string()),
        visible_assets: vec![AssetItemSnapshot {
            uuid: "asset-a".to_string(),
            locator: "res://ui/editor/workbench_page_chrome.zui".to_string(),
            display_name: "workbench_page_chrome.zui".to_string(),
            file_name: "workbench_page_chrome.zui".to_string(),
            extension: "zui".to_string(),
            kind: ResourceKind::UiLayout,
            asset_type:
                crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
                    ResourceKind::UiLayout,
                ),
            preview_artifact_path: String::new(),
            dirty: false,
            diagnostics: Vec::new(),
            selected: false,
            resource_state: None,
            resource_revision: Some(42),
        }]
        .into(),
        ..AssetWorkspaceSnapshot::default()
    };
    let mut nodes = vec![ViewTemplateNodeData {
        node_id: "asset_browser.content_preview.name".into(),
        control_id: "AssetBrowserContentPreviewName".into(),
        role: "Label".into(),
        text: "workbench_page_chrome.zui".into(),
        frame: ViewTemplateFrameData::default(),
        ..ViewTemplateNodeData::default()
    }];

    append_asset_browser_summary_nodes(&mut nodes, &snapshot);

    let name = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserContentPreviewName")
        .expect("summary name node should exist");
    let continuation = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserContentPreviewNameContinuation")
        .expect("summary continuation node should exist");
    assert_eq!(name.text.as_str(), "workbench_page_chrome.zui");
    assert_eq!(continuation.text.as_str(), "");
    assert_eq!(continuation.frame.height, 0.0);
}

#[test]
fn summary_file_like_title_uses_runtime_width_not_character_count() {
    let narrow = format!("{}.zui", "i".repeat(44));
    let wide = format!("{}.zui", "W".repeat(44));
    assert_eq!(narrow.chars().count(), wide.chars().count());

    assert_eq!(summary_file_like_display_title(&narrow, "zui"), narrow);
    let compact_wide = summary_file_like_display_title(&wide, "zui");

    assert_ne!(compact_wide, wide);
    assert!(compact_wide.ends_with(".zui"));
    assert!(
        measure_runtime_text_width(&compact_wide, SUMMARY_NAME_FONT_SIZE)
            <= SUMMARY_FILE_NAME_MAX_WIDTH + 0.01,
        "summary file-like title should fit measured width: {compact_wide}"
    );
}

#[test]
fn summary_nodes_remove_inline_summary_for_thumbnail_view() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        ..AssetWorkspaceSnapshot::default()
    };
    let mut nodes = vec![
        ViewTemplateNodeData {
            control_id: "AssetBrowserContentPreviewCard".into(),
            ..ViewTemplateNodeData::default()
        },
        ViewTemplateNodeData {
            control_id: "AssetBrowserContentPreviewName".into(),
            ..ViewTemplateNodeData::default()
        },
        ViewTemplateNodeData {
            control_id: "AssetBrowserContentPanel".into(),
            ..ViewTemplateNodeData::default()
        },
    ];

    sync_asset_browser_summary_nodes(&mut nodes, &snapshot);

    assert!(nodes
        .iter()
        .all(|node| node.control_id != "AssetBrowserContentPreviewCard"));
    assert!(nodes
        .iter()
        .all(|node| node.control_id != "AssetBrowserContentPreviewName"));
    assert!(nodes
        .iter()
        .any(|node| node.control_id == "AssetBrowserContentPanel"));
}

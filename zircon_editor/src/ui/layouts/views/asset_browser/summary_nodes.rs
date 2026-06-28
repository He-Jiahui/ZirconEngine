use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetViewMode, AssetWorkspaceSnapshot};

use super::thumbnail_nodes::asset_display_name_lines;
use super::{asset_state_label, compact_resource_kind_label};

const SUMMARY_NAME_CONTROL_ID: &str = "AssetBrowserContentPreviewName";
const SUMMARY_NAME_CONTINUATION_CONTROL_ID: &str = "AssetBrowserContentPreviewNameContinuation";
const SUMMARY_TYPE_BADGE_CONTROL_ID: &str = "AssetBrowserContentPreviewTypeBadge";
const SUMMARY_TYPE_CONTROL_ID: &str = "AssetBrowserContentPreviewType";
const SUMMARY_STATE_CONTROL_ID: &str = "AssetBrowserContentPreviewState";
const SUMMARY_REVISION_CONTROL_ID: &str = "AssetBrowserContentPreviewRevision";
const SUMMARY_NAME_FONT_SIZE: f32 = 10.0;
const SUMMARY_NAME_FONT_WEIGHT: i32 = 600;
const SUMMARY_NAME_CONTINUATION_FONT_SIZE: f32 = 9.0;
const SUMMARY_NAME_CONTINUATION_FONT_WEIGHT: i32 = 500;

pub(super) fn sync_asset_browser_summary_nodes(
    nodes: &mut Vec<ViewTemplateNodeData>,
    snapshot: &AssetWorkspaceSnapshot,
) {
    if snapshot.view_mode == AssetViewMode::Thumbnail {
        remove_content_preview_summary_nodes(nodes);
        return;
    }

    append_asset_browser_summary_nodes(nodes, snapshot);
}

pub(super) fn append_asset_browser_summary_nodes(
    nodes: &mut Vec<ViewTemplateNodeData>,
    snapshot: &AssetWorkspaceSnapshot,
) {
    let selected_asset = selected_asset(snapshot);
    let (name, name_continuation) = selected_asset
        .map(|asset| asset_display_name_lines(asset.display_name.as_str()))
        .unwrap_or_else(|| ("No Asset Selected".to_string(), String::new()));
    update_summary_name_node(nodes, name);
    nodes.push(summary_label_node(
        "asset_browser.content_preview.name_continuation",
        SUMMARY_NAME_CONTINUATION_CONTROL_ID,
        name_continuation,
        SUMMARY_NAME_CONTINUATION_FONT_SIZE,
        SUMMARY_NAME_CONTINUATION_FONT_WEIGHT,
        "muted",
    ));
    nodes.push(summary_type_badge_node());
    nodes.push(summary_label_node(
        "asset_browser.content_preview.type",
        SUMMARY_TYPE_CONTROL_ID,
        selected_asset.map(summary_type_label).unwrap_or_default(),
        8.0,
        700,
        "accent",
    ));
    nodes.push(summary_label_node(
        "asset_browser.content_preview.state",
        SUMMARY_STATE_CONTROL_ID,
        selected_asset
            .map(asset_state_label)
            .unwrap_or("Select asset")
            .to_string(),
        9.0,
        400,
        "muted",
    ));
    nodes.push(summary_label_node(
        "asset_browser.content_preview.revision",
        SUMMARY_REVISION_CONTROL_ID,
        selected_asset
            .map(summary_revision_label)
            .unwrap_or_default(),
        9.0,
        400,
        "muted",
    ));
}

fn update_summary_name_node(nodes: &mut [ViewTemplateNodeData], text: String) {
    for node in nodes
        .iter_mut()
        .filter(|node| node.control_id.as_str() == SUMMARY_NAME_CONTROL_ID)
    {
        node.text = text.clone().into();
        node.overflow = "elide".into();
        node.font_size = SUMMARY_NAME_FONT_SIZE;
        node.font_weight = SUMMARY_NAME_FONT_WEIGHT;
    }
}

fn remove_content_preview_summary_nodes(nodes: &mut Vec<ViewTemplateNodeData>) {
    nodes.retain(|node| !is_content_preview_summary_node(node.control_id.as_str()));
}

fn is_content_preview_summary_node(control_id: &str) -> bool {
    matches!(
        control_id,
        "AssetBrowserContentPreviewCard"
            | "AssetBrowserContentPreviewVisual"
            | "AssetBrowserContentPreviewName"
            | "AssetBrowserContentPreviewNameContinuation"
            | "AssetBrowserContentPreviewMeta"
            | "AssetBrowserContentPreviewTypeBadge"
            | "AssetBrowserContentPreviewType"
            | "AssetBrowserContentPreviewState"
            | "AssetBrowserContentPreviewRevision"
    )
}

fn selected_asset(snapshot: &AssetWorkspaceSnapshot) -> Option<&AssetItemSnapshot> {
    let selected_uuid = snapshot.selected_asset_uuid.as_deref();
    snapshot
        .visible_assets
        .iter()
        .find(|asset| selected_uuid == Some(asset.uuid.as_str()) || asset.selected)
}

fn summary_type_label(asset: &AssetItemSnapshot) -> String {
    compact_resource_kind_label(asset.kind).to_ascii_uppercase()
}

fn summary_revision_label(asset: &AssetItemSnapshot) -> String {
    asset
        .resource_revision
        .map(|revision| format!("rev {revision}"))
        .unwrap_or_else(|| "untracked".to_string())
}

fn summary_type_badge_node() -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: "asset_browser.content_preview.type_badge".into(),
        control_id: SUMMARY_TYPE_BADGE_CONTROL_ID.into(),
        role: "Panel".into(),
        surface_variant: "asset-type-badge".into(),
        corner_radius: 3.0,
        frame: ViewTemplateFrameData::default(),
        ..ViewTemplateNodeData::default()
    }
}

fn summary_label_node(
    node_id: &str,
    control_id: &str,
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

#[cfg(test)]
mod tests {
    use super::*;
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
                preview_artifact_path: String::new(),
                dirty: false,
                diagnostics: Vec::new(),
                selected: false,
                resource_state: None,
                resource_revision: Some(42),
            }],
            ..AssetWorkspaceSnapshot::default()
        };
        let mut nodes = Vec::new();

        append_asset_browser_summary_nodes(&mut nodes, &snapshot);

        assert!(nodes.iter().any(|node| {
            node.control_id == SUMMARY_TYPE_BADGE_CONTROL_ID
                && node.role == "Panel"
                && node.surface_variant == "asset-type-badge"
        }));
        assert!(nodes.iter().any(|node| {
            node.control_id == SUMMARY_TYPE_CONTROL_ID
                && node.text == "UI"
                && node.text_tone == "accent"
                && node.font_size == 8.0
                && node.font_weight == 700
        }));
        assert!(nodes.iter().any(|node| {
            node.control_id == SUMMARY_STATE_CONTROL_ID
                && node.text == "Ready"
                && node.text_tone == "muted"
        }));
        assert!(nodes.iter().any(|node| {
            node.control_id == SUMMARY_REVISION_CONTROL_ID && node.text == "rev 42"
        }));
    }

    #[test]
    fn summary_nodes_split_selected_long_name_into_primary_and_continuation_labels() {
        let snapshot = AssetWorkspaceSnapshot {
            selected_asset_uuid: Some("asset-a".to_string()),
            visible_assets: vec![AssetItemSnapshot {
                uuid: "asset-a".to_string(),
                locator: "res://a".to_string(),
                display_name: "workbench_host_window.zui".to_string(),
                file_name: "workbench_host_window.zui".to_string(),
                extension: "zui".to_string(),
                kind: ResourceKind::UiLayout,
                preview_artifact_path: String::new(),
                dirty: false,
                diagnostics: Vec::new(),
                selected: false,
                resource_state: None,
                resource_revision: Some(42),
            }],
            ..AssetWorkspaceSnapshot::default()
        };
        let mut nodes = vec![ViewTemplateNodeData {
            node_id: "asset_browser.content_preview.name".into(),
            control_id: "AssetBrowserContentPreviewName".into(),
            role: "Label".into(),
            text: "workbench_host_window.zui".into(),
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
        assert_eq!(name.text.as_str(), "workbench_host");
        assert_eq!(name.font_size, 10.0);
        assert_eq!(name.font_weight, 600);
        assert_eq!(continuation.text.as_str(), "window.zui");
        assert_eq!(continuation.role.as_str(), "Label");
        assert_eq!(continuation.font_size, 9.0);
        assert_eq!(continuation.font_weight, 500);
        assert_eq!(continuation.text_tone.as_str(), "muted");
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
}

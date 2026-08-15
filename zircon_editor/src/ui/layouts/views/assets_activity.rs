use std::collections::BTreeMap;

use crate::ui::layouts::views::view_projection::{
    AssetWorkspaceProjectionGeneration, build_view_template_node_projection,
    compose_view_template_node_model,
};
use crate::ui::layouts::windows::workbench_host_window::AssetsActivityPaneViewData;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::{
    AssetContentPaintNodeInput, AssetContentSurface, asset_content_paint_metadata,
};
use crate::ui::workbench::snapshot::{AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot};
use zircon_runtime_interface::ui::layout::UiSize;

mod content_layout;
mod content_nodes;
mod reference_nodes;
mod responsive_layout;
#[cfg(test)]
mod responsive_layout_tests;

use content_layout::apply_assets_activity_content_layout;
use content_nodes::append_assets_activity_content_nodes;
use reference_nodes::{
    apply_assets_activity_reference_layout, sync_assets_activity_reference_nodes,
};
use responsive_layout::apply_assets_activity_responsive_layout;

const ASSETS_ACTIVITY_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/assets_activity.zui";
const ASSETS_ACTIVITY_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.zui";
const ASSETS_ACTIVITY_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.zui";

pub(crate) fn assets_activity_pane_data(
    snapshot: &AssetWorkspaceSnapshot,
    size: UiSize,
) -> AssetsActivityPaneViewData {
    let mut text_overrides = BTreeMap::new();
    let project_root = if snapshot.project_root.is_empty() {
        "Browse project assets".to_string()
    } else {
        snapshot.project_root.clone()
    };
    let selection_text = if snapshot.selection.display_name.is_empty() {
        "Select an asset to inspect".to_string()
    } else {
        snapshot.selection.display_name.clone()
    };
    let selection_locator = if snapshot.selection.locator.is_empty() {
        "No project locator".to_string()
    } else {
        snapshot.selection.locator.clone()
    };
    let selection_kind = if snapshot.selection.asset_type.display_name.is_empty() {
        "Unknown Type".to_string()
    } else {
        snapshot.selection.asset_type.display_name.clone()
    };
    let selection_identity = snapshot
        .selection
        .uuid
        .clone()
        .unwrap_or_else(|| "No UUID".to_string());
    let selection_toolkit = if snapshot.selection.toolkit_view_id.is_empty() {
        "No toolkit".to_string()
    } else {
        snapshot.selection.toolkit_view_id.clone()
    };
    let selection_meta_path = if snapshot.selection.meta_path.is_empty() {
        "No meta path".to_string()
    } else {
        snapshot.selection.meta_path.clone()
    };
    let selection_diagnostics = if snapshot.selection.diagnostics.is_empty() {
        "No active diagnostics".to_string()
    } else {
        snapshot.selection.diagnostics.join("\n")
    };

    text_overrides.insert(
        "AssetsActivitySubtitleText".to_string(),
        project_root.clone(),
    );
    text_overrides.insert(
        "AssetsActivityTreeSubtitleText".to_string(),
        project_root.clone(),
    );
    text_overrides.insert(
        "AssetsActivityTreeRowNameText".to_string(),
        snapshot
            .selected_folder_id
            .clone()
            .unwrap_or_else(|| "Assets".to_string()),
    );
    text_overrides.insert(
        "AssetsActivityTreeRowCountText".to_string(),
        snapshot.visible_assets.len().to_string(),
    );
    text_overrides.insert("AssetsActivitySelectionText".to_string(), selection_text);
    text_overrides.insert(
        "AssetsActivityPreviewNameText".to_string(),
        if snapshot.selection.display_name.is_empty() {
            "No Asset Selected".to_string()
        } else {
            snapshot.selection.display_name.clone()
        },
    );
    text_overrides.insert(
        "AssetsActivityPreviewLocatorText".to_string(),
        selection_locator,
    );
    text_overrides.insert("AssetsActivityPreviewKindText".to_string(), selection_kind);
    text_overrides.insert(
        "AssetsActivityPreviewIdentityText".to_string(),
        selection_identity,
    );
    text_overrides.insert(
        "AssetsActivityPreviewToolkitText".to_string(),
        selection_toolkit,
    );
    text_overrides.insert(
        "AssetsActivityPreviewMetaPathText".to_string(),
        selection_meta_path,
    );
    text_overrides.insert(
        "AssetsActivityPreviewDiagnosticsText".to_string(),
        selection_diagnostics,
    );
    text_overrides.insert(
        "AssetsActivityViewModeListButton".to_string(),
        "List".to_string(),
    );
    text_overrides.insert(
        "AssetsActivityViewModeThumbButton".to_string(),
        "Thumb".to_string(),
    );
    text_overrides.insert("AssetsActivityKindAllChip".to_string(), "All".to_string());
    text_overrides.insert(
        "AssetsActivityKindTextureChip".to_string(),
        "Tex".to_string(),
    );
    text_overrides.insert(
        "AssetsActivityKindMaterialChip".to_string(),
        "Mat".to_string(),
    );
    text_overrides.insert("AssetsActivityKindSceneChip".to_string(), "Scn".to_string());
    text_overrides.insert(
        "AssetsActivityKindModelChip".to_string(),
        "Mesh".to_string(),
    );
    text_overrides.insert(
        "AssetsActivityKindShaderChip".to_string(),
        "Shd".to_string(),
    );
    text_overrides.insert(
        "AssetsActivityPreviewTabButton".to_string(),
        "Preview".to_string(),
    );
    text_overrides.insert(
        "AssetsActivityReferencesTabButton".to_string(),
        "References".to_string(),
    );
    text_overrides.insert("SearchEdited".to_string(), snapshot.search_query.clone());

    let Ok(projection) = build_view_template_node_projection(
        "assets_activity.template_projection",
        ASSETS_ACTIVITY_LAYOUT_ASSET_PATH,
        &[(
            ASSETS_ACTIVITY_STYLE_ASSET_ID,
            ASSETS_ACTIVITY_STYLE_ASSET_PATH,
        )],
        size,
        &text_overrides,
    ) else {
        return AssetsActivityPaneViewData {
            nodes: ModelRc::default(),
        };
    };
    let generation = AssetWorkspaceProjectionGeneration::from_snapshot(snapshot);
    let nodes = compose_view_template_node_model(
        "assets_activity.template_composition",
        projection,
        &generation,
        |nodes| {
            append_assets_activity_content_nodes(nodes, snapshot);
            if snapshot.utility_tab == AssetUtilityTab::References {
                sync_assets_activity_reference_nodes(nodes, snapshot);
            }
            apply_assets_activity_visual_state(nodes, snapshot);
            apply_assets_activity_responsive_layout(nodes, snapshot, size);
            if snapshot.utility_tab == AssetUtilityTab::References {
                apply_assets_activity_reference_layout(nodes);
            }
            apply_assets_activity_content_layout(nodes, snapshot);

            asset_content_paint_metadata(
                nodes.iter().map(|node| {
                    AssetContentPaintNodeInput::new(
                        node.control_id.as_str(),
                        node.frame.x,
                        node.frame.y,
                        node.frame.width,
                        node.frame.height,
                        node.value_number,
                    )
                }),
                AssetContentSurface::Activity,
            )
        },
    );
    AssetsActivityPaneViewData { nodes }
}

#[cfg(test)]
#[test]
fn stable_assets_activity_snapshot_reuses_the_composed_model() {
    super::view_projection::clear_view_template_projection_caches_for_tests();
    let snapshot = AssetWorkspaceSnapshot::default();
    let size = UiSize::new(420.0, 360.0);

    let first = assets_activity_pane_data(&snapshot, size);
    let stable = assets_activity_pane_data(&snapshot, size);

    assert!(first.nodes.shares_values_with(&stable.nodes));
}

fn apply_assets_activity_visual_state(
    nodes: &mut [crate::ui::layouts::views::ViewTemplateNodeData],
    snapshot: &AssetWorkspaceSnapshot,
) {
    mark_toggle_state(
        nodes,
        "AssetsActivityViewModeListButton",
        snapshot.view_mode == AssetViewMode::List,
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityViewModeThumbButton",
        snapshot.view_mode == AssetViewMode::Thumbnail,
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityPreviewTabButton",
        snapshot.utility_tab == AssetUtilityTab::Preview,
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityReferencesTabButton",
        snapshot.utility_tab == AssetUtilityTab::References,
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityKindAllChip",
        snapshot.kind_filter.is_none(),
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityKindTextureChip",
        snapshot.kind_filter == Some(zircon_runtime_interface::resource::ResourceKind::Texture),
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityKindMaterialChip",
        snapshot.kind_filter == Some(zircon_runtime_interface::resource::ResourceKind::Material),
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityKindSceneChip",
        snapshot.kind_filter == Some(zircon_runtime_interface::resource::ResourceKind::Scene),
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityKindModelChip",
        snapshot.kind_filter == Some(zircon_runtime_interface::resource::ResourceKind::Model),
    );
    mark_toggle_state(
        nodes,
        "AssetsActivityKindShaderChip",
        snapshot.kind_filter == Some(zircon_runtime_interface::resource::ResourceKind::Shader),
    );
    let has_selection =
        snapshot.selected_asset_uuid.is_some() || !snapshot.selection.display_name.is_empty();
    mark_panel_selected(
        nodes,
        "AssetsActivityTreeRowPanel",
        snapshot.selected_folder_id.is_some(),
    );
    mark_panel_selected(
        nodes,
        "AssetsActivityPreviewPanel",
        has_selection && snapshot.utility_tab == AssetUtilityTab::Preview,
    );
    mark_panel_selected(
        nodes,
        "AssetsActivityPreviewVisualPanel",
        has_selection && snapshot.utility_tab == AssetUtilityTab::Preview,
    );
    mark_panel_group_selected(
        nodes,
        &[
            "AssetsActivityReferenceLeftPanel",
            "AssetsActivityReferenceRightPanel",
        ],
        snapshot.utility_tab == AssetUtilityTab::References,
    );
    mark_text_state(
        nodes,
        &[
            "AssetsActivityPreviewNameText",
            "AssetsActivityPreviewLocatorText",
            "AssetsActivityPreviewKindText",
            "AssetsActivityPreviewIdentityText",
            "AssetsActivityPreviewToolkitText",
            "AssetsActivityPreviewMetaPathText",
            "AssetsActivityPreviewDiagnosticsText",
        ],
        has_selection && snapshot.utility_tab == AssetUtilityTab::Preview,
    );
}

fn mark_toggle_state(
    nodes: &mut [crate::ui::layouts::views::ViewTemplateNodeData],
    control_id: &str,
    active: bool,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = active;
        node.focused = false;
        node.surface_variant = if active { "inset".into() } else { "".into() };
        node.text_tone = if active {
            "default".into()
        } else {
            "subtle".into()
        };
    }
}

fn mark_panel_selected(
    nodes: &mut [crate::ui::layouts::views::ViewTemplateNodeData],
    control_id: &str,
    selected: bool,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = selected;
        node.focused = false;
    }
}

fn mark_panel_group_selected(
    nodes: &mut [crate::ui::layouts::views::ViewTemplateNodeData],
    control_ids: &[&str],
    selected: bool,
) {
    for control_id in control_ids {
        mark_panel_selected(nodes, control_id, selected);
    }
}

fn mark_text_state(
    nodes: &mut [crate::ui::layouts::views::ViewTemplateNodeData],
    control_ids: &[&str],
    active: bool,
) {
    for control_id in control_ids {
        if let Some(node) = nodes.iter_mut().find(|node| node.control_id == *control_id) {
            node.selected = active;
            node.focused = false;
            node.text_tone = if active {
                "default".into()
            } else {
                "muted".into()
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{mark_panel_selected, mark_text_state, mark_toggle_state};
    use crate::ui::layouts::views::ViewTemplateNodeData;

    #[test]
    fn visual_selection_state_does_not_impersonate_keyboard_focus() {
        let mut nodes = vec![node("toggle"), node("panel"), node("label")];

        mark_toggle_state(&mut nodes, "toggle", true);
        mark_panel_selected(&mut nodes, "panel", true);
        mark_text_state(&mut nodes, &["label"], true);

        assert!(nodes.iter().all(|node| node.selected));
        assert!(nodes.iter().all(|node| !node.focused));
    }

    fn node(control_id: &str) -> ViewTemplateNodeData {
        ViewTemplateNodeData {
            control_id: control_id.into(),
            ..ViewTemplateNodeData::default()
        }
    }
}

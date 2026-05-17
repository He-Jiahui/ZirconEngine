use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::view_projection::build_view_template_nodes;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::snapshot::{
    AssetFolderSnapshot, AssetSelectionSnapshot, AssetUtilityTab, AssetViewMode,
    AssetWorkspaceSnapshot,
};
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const ASSET_BROWSER_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/asset_browser.v2.ui.toml";
const ASSET_BROWSER_MATERIAL_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_material.v2.ui.toml";
const ASSET_BROWSER_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.v2.ui.toml";
const ASSET_BROWSER_MATERIAL_STYLE_ASSET_ID: &str = "res://ui/theme/editor_material.v2.ui.toml";
const ASSET_BROWSER_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.v2.ui.toml";

pub(crate) fn asset_browser_pane_nodes(
    snapshot: &AssetWorkspaceSnapshot,
    size: UiSize,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = BTreeMap::new();
    let project_root = if snapshot.project_root.is_empty() {
        "Project/assets".to_string()
    } else {
        snapshot.project_root.clone()
    };
    let selected_folder = selected_folder(snapshot);
    let selection_name = if snapshot.selection.display_name.is_empty() {
        "No Asset Selected".to_string()
    } else {
        snapshot.selection.display_name.clone()
    };
    let selection_locator = if snapshot.selection.locator.is_empty() {
        "Select an asset to inspect".to_string()
    } else {
        snapshot.selection.locator.clone()
    };
    let selection_kind = snapshot
        .selection
        .kind
        .map(resource_kind_label)
        .unwrap_or("Unknown Type")
        .to_string();
    let selection_identity = snapshot
        .selection
        .uuid
        .clone()
        .unwrap_or_else(|| "No UUID".to_string());
    let selection_revision = snapshot
        .selection
        .resource_revision
        .map(|revision| format!("Revision {revision}"))
        .unwrap_or_else(|| "Revision pending".to_string());
    let selection_diagnostics = if snapshot.selection.diagnostics.is_empty() {
        "No active diagnostics".to_string()
    } else {
        snapshot.selection.diagnostics.join("\n")
    };
    let selection_metadata_summary = selection_metadata_summary(&snapshot.selection);
    let selection_metadata_body =
        selection_metadata_body(&snapshot.selection, &selection_diagnostics);
    let catalog_summary = format!(
        "{} folders • {} assets",
        snapshot.visible_folders.len(),
        snapshot.visible_assets.len()
    );

    text_overrides.insert("AssetBrowserSubtitleText".to_string(), project_root.clone());
    text_overrides.insert(
        "AssetBrowserSourcesSubtitleText".to_string(),
        project_root.clone(),
    );
    text_overrides.insert(
        "AssetBrowserSourcesRowNameText".to_string(),
        selected_folder
            .map(|folder| folder.display_name.clone())
            .unwrap_or_else(|| project_root.clone()),
    );
    text_overrides.insert(
        "AssetBrowserSourcesRowCountText".to_string(),
        selected_folder
            .map(|folder| folder.recursive_asset_count)
            .unwrap_or(snapshot.visible_assets.len())
            .to_string(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsHeaderSelectionText".to_string(),
        selection_locator.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsPreviewNameText".to_string(),
        selection_name.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsPreviewLocatorText".to_string(),
        selection_locator.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsPreviewKindText".to_string(),
        selection_kind.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsPreviewIdentityText".to_string(),
        selection_identity.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsPreviewAdapterText".to_string(),
        if snapshot.selection.adapter_key.is_empty() {
            "No adapter".to_string()
        } else {
            snapshot.selection.adapter_key.clone()
        },
    );
    text_overrides.insert(
        "AssetBrowserDetailsPreviewMetaPathText".to_string(),
        if snapshot.selection.meta_path.is_empty() {
            "No meta path".to_string()
        } else {
            snapshot.selection.meta_path.clone()
        },
    );
    text_overrides.insert(
        "AssetBrowserDetailsPreviewDiagnosticsText".to_string(),
        selection_diagnostics.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsLocatorValue".to_string(),
        selection_locator.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsTypeValue".to_string(),
        selection_kind.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsIdentityUuidValue".to_string(),
        selection_identity,
    );
    text_overrides.insert(
        "AssetBrowserDetailsIdentityRevisionValue".to_string(),
        selection_revision,
    );
    text_overrides.insert(
        "AssetBrowserDetailsMetadataMetaPathValue".to_string(),
        if snapshot.selection.meta_path.is_empty() {
            "No meta path".to_string()
        } else {
            snapshot.selection.meta_path.clone()
        },
    );
    text_overrides.insert(
        "AssetBrowserDetailsMetadataAdapterValue".to_string(),
        selection_metadata_summary.clone(),
    );
    text_overrides.insert(
        "AssetBrowserDetailsDiagnosticsText".to_string(),
        selection_diagnostics.clone(),
    );
    text_overrides.insert(
        "AssetBrowserSelectionLocatorText".to_string(),
        selection_locator.clone(),
    );
    text_overrides.insert("AssetBrowserPreviewNameText".to_string(), selection_name);
    text_overrides.insert(
        "AssetBrowserPreviewLocatorText".to_string(),
        selection_locator,
    );
    text_overrides.insert("AssetBrowserPreviewKindText".to_string(), selection_kind);
    text_overrides.insert(
        "AssetBrowserPreviewIdentityText".to_string(),
        snapshot
            .selection
            .uuid
            .clone()
            .unwrap_or_else(|| "No UUID".to_string()),
    );
    text_overrides.insert(
        "AssetBrowserPreviewAdapterText".to_string(),
        if snapshot.selection.adapter_key.is_empty() {
            "No adapter".to_string()
        } else {
            snapshot.selection.adapter_key.clone()
        },
    );
    text_overrides.insert(
        "AssetBrowserPreviewMetaPathText".to_string(),
        if snapshot.selection.meta_path.is_empty() {
            "No meta path".to_string()
        } else {
            snapshot.selection.meta_path.clone()
        },
    );
    text_overrides.insert(
        "AssetBrowserPreviewDiagnosticsText".to_string(),
        selection_diagnostics.clone(),
    );
    text_overrides.insert(
        "AssetBrowserReferenceLeftEmptyText".to_string(),
        if snapshot.selection.references.is_empty() {
            "No direct references".to_string()
        } else {
            format!("{} direct references", snapshot.selection.references.len())
        },
    );
    text_overrides.insert(
        "AssetBrowserReferenceRightEmptyText".to_string(),
        if snapshot.selection.used_by.is_empty() {
            "No direct references".to_string()
        } else {
            format!("{} usages", snapshot.selection.used_by.len())
        },
    );
    text_overrides.insert(
        "AssetBrowserMetaPathValue".to_string(),
        if snapshot.selection.meta_path.is_empty() {
            "No meta path".to_string()
        } else {
            snapshot.selection.meta_path.clone()
        },
    );
    text_overrides.insert(
        "AssetBrowserAdapterValue".to_string(),
        selection_metadata_summary,
    );
    text_overrides.insert(
        "AssetBrowserAdapterLabel".to_string(),
        "Adapter / Package".to_string(),
    );
    text_overrides.insert(
        "AssetBrowserDiagnosticsText".to_string(),
        selection_metadata_body,
    );
    text_overrides.insert(
        "AssetBrowserDiagnosticsLabel".to_string(),
        "Diagnostics / Subassets".to_string(),
    );
    text_overrides.insert(
        "AssetBrowserPluginsText".to_string(),
        format!(
            "{} • view {:?} • tab {:?}",
            catalog_summary, snapshot.view_mode, snapshot.utility_tab
        ),
    );
    text_overrides.insert(
        "AssetBrowserViewModeListButton".to_string(),
        "List".to_string(),
    );
    text_overrides.insert(
        "AssetBrowserViewModeThumbButton".to_string(),
        "Thumb".to_string(),
    );
    text_overrides.insert("AssetBrowserKindAllChip".to_string(), "All".to_string());
    text_overrides.insert(
        "AssetBrowserKindTextureChip".to_string(),
        "Texture".to_string(),
    );
    text_overrides.insert(
        "AssetBrowserKindMaterialChip".to_string(),
        "Material".to_string(),
    );
    text_overrides.insert("AssetBrowserKindSceneChip".to_string(), "Scene".to_string());
    text_overrides.insert("AssetBrowserKindModelChip".to_string(), "Model".to_string());
    text_overrides.insert(
        "AssetBrowserKindShaderChip".to_string(),
        "Shader".to_string(),
    );
    text_overrides.insert(
        "AssetBrowserPreviewTabButton".to_string(),
        "Preview".to_string(),
    );
    text_overrides.insert(
        "AssetBrowserReferencesTabButton".to_string(),
        "References".to_string(),
    );
    text_overrides.insert(
        "AssetBrowserMetadataTabButton".to_string(),
        "Metadata".to_string(),
    );
    text_overrides.insert(
        "AssetBrowserPluginsTabButton".to_string(),
        "Plugins".to_string(),
    );
    text_overrides.insert(
        "SearchEdited".to_string(),
        if snapshot.search_query.is_empty() {
            "Search".to_string()
        } else {
            snapshot.search_query.clone()
        },
    );

    let mut nodes = build_view_template_nodes(
        "asset_browser.template_projection",
        ASSET_BROWSER_LAYOUT_ASSET_PATH,
        &[
            (
                ASSET_BROWSER_MATERIAL_STYLE_ASSET_ID,
                ASSET_BROWSER_MATERIAL_STYLE_ASSET_PATH,
            ),
            (ASSET_BROWSER_STYLE_ASSET_ID, ASSET_BROWSER_STYLE_ASSET_PATH),
        ],
        size,
        &text_overrides,
    )
    .unwrap_or_default();
    apply_asset_browser_visual_state(&mut nodes, snapshot);
    model_rc(nodes)
}

fn apply_asset_browser_visual_state(
    nodes: &mut [ViewTemplateNodeData],
    snapshot: &AssetWorkspaceSnapshot,
) {
    mark_toggle_state(
        nodes,
        "AssetBrowserViewModeListButton",
        snapshot.view_mode == AssetViewMode::List,
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserViewModeThumbButton",
        snapshot.view_mode == AssetViewMode::Thumbnail,
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserPreviewTabButton",
        snapshot.utility_tab == AssetUtilityTab::Preview,
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserReferencesTabButton",
        snapshot.utility_tab == AssetUtilityTab::References,
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserMetadataTabButton",
        snapshot.utility_tab == AssetUtilityTab::Metadata,
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserPluginsTabButton",
        snapshot.utility_tab == AssetUtilityTab::Plugins,
    );

    mark_toggle_state(
        nodes,
        "AssetBrowserKindAllChip",
        snapshot.kind_filter.is_none(),
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserKindTextureChip",
        snapshot.kind_filter == Some(ResourceKind::Texture),
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserKindMaterialChip",
        snapshot.kind_filter == Some(ResourceKind::Material),
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserKindSceneChip",
        snapshot.kind_filter == Some(ResourceKind::Scene),
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserKindModelChip",
        snapshot.kind_filter == Some(ResourceKind::Model),
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserKindShaderChip",
        snapshot.kind_filter == Some(ResourceKind::Shader),
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserKindPhysicsChip",
        snapshot.kind_filter == Some(ResourceKind::PhysicsMaterial),
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserKindSkeletonChip",
        snapshot.kind_filter == Some(ResourceKind::AnimationSkeleton),
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserKindClipChip",
        snapshot.kind_filter == Some(ResourceKind::AnimationClip),
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserKindSequenceChip",
        snapshot.kind_filter == Some(ResourceKind::AnimationSequence),
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserKindGraphChip",
        snapshot.kind_filter == Some(ResourceKind::AnimationGraph),
    );
    mark_toggle_state(
        nodes,
        "AssetBrowserKindStateChip",
        snapshot.kind_filter == Some(ResourceKind::AnimationStateMachine),
    );

    let has_selection =
        snapshot.selected_asset_uuid.is_some() || !snapshot.selection.display_name.is_empty();
    mark_panel_selected(
        nodes,
        "AssetBrowserSourcesRowPanel",
        snapshot.selected_folder_id.is_some(),
    );
    mark_panel_selected(nodes, "AssetBrowserDetailsHeaderPanel", has_selection);
    mark_panel_selected(nodes, "AssetBrowserDetailsPreviewPanel", has_selection);
    mark_panel_selected(nodes, "AssetBrowserDetailsLocatorPanel", has_selection);
    mark_panel_selected(nodes, "AssetBrowserDetailsTypePanel", has_selection);
    mark_panel_selected(nodes, "AssetBrowserDetailsIdentityPanel", has_selection);
    mark_panel_selected(nodes, "AssetBrowserDetailsMetadataPanel", has_selection);
    mark_panel_selected(
        nodes,
        "AssetBrowserPreviewPanel",
        has_selection && snapshot.utility_tab == AssetUtilityTab::Preview,
    );
    mark_panel_group_selected(
        nodes,
        &[
            "AssetBrowserReferenceLeftPanel",
            "AssetBrowserReferenceRightPanel",
            "AssetBrowserReferenceLeftRowPanel",
            "AssetBrowserReferenceRightRowPanel",
        ],
        snapshot.utility_tab == AssetUtilityTab::References,
    );
    mark_panel_group_selected(
        nodes,
        &[
            "AssetBrowserMetaPathPanel",
            "AssetBrowserAdapterPanel",
            "AssetBrowserDiagnosticsPanel",
            "AssetBrowserDetailsMetadataPanel",
            "AssetBrowserDetailsDiagnosticsPanel",
        ],
        snapshot.utility_tab == AssetUtilityTab::Metadata,
    );
    mark_panel_selected(
        nodes,
        "AssetBrowserPluginsPanel",
        snapshot.utility_tab == AssetUtilityTab::Plugins,
    );

    if snapshot.selection.diagnostics.is_empty() {
        update_panel_variant(nodes, "AssetBrowserDetailsDiagnosticsPanel", "inset");
    }
}

fn selected_folder(snapshot: &AssetWorkspaceSnapshot) -> Option<&AssetFolderSnapshot> {
    let selected_folder_id = snapshot.selected_folder_id.as_deref()?;
    snapshot
        .visible_folders
        .iter()
        .chain(snapshot.folder_tree.iter())
        .find(|folder| folder.folder_id == selected_folder_id)
}

fn mark_toggle_state(nodes: &mut [ViewTemplateNodeData], control_id: &str, active: bool) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = active;
        node.focused = active;
        node.surface_variant = if active { "inset".into() } else { "".into() };
        node.text_tone = if active {
            "default".into()
        } else {
            "subtle".into()
        };
    }
}

fn mark_panel_selected(nodes: &mut [ViewTemplateNodeData], control_id: &str, selected: bool) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = selected;
        node.focused = selected;
    }
}

fn mark_panel_group_selected(
    nodes: &mut [ViewTemplateNodeData],
    control_ids: &[&str],
    selected: bool,
) {
    for control_id in control_ids {
        mark_panel_selected(nodes, control_id, selected);
    }
}

fn update_panel_variant(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    surface_variant: &str,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.surface_variant = surface_variant.into();
    }
}

fn selection_metadata_summary(selection: &AssetSelectionSnapshot) -> String {
    let mut parts = Vec::new();
    parts.push(if selection.adapter_key.is_empty() {
        "No adapter".to_string()
    } else {
        selection.adapter_key.clone()
    });
    if !selection.asset_unit.is_empty() {
        parts.push(format!("unit {}", selection.asset_unit));
    }
    if let Some(package_id) = selection.package_id.as_ref() {
        parts.push(format!("package {package_id}"));
    }
    if !selection.included_files.is_empty() {
        parts.push(format!("{} included", selection.included_files.len()));
    }
    if !selection.subassets.is_empty() {
        parts.push(format!("{} subassets", selection.subassets.len()));
    }
    parts.join(" | ")
}

fn selection_metadata_body(selection: &AssetSelectionSnapshot, diagnostics: &str) -> String {
    let mut lines = Vec::new();
    if selection.diagnostics.is_empty() {
        lines.push("No active diagnostics".to_string());
    } else {
        lines.push(diagnostics.to_string());
    }
    if !selection.included_files.is_empty() {
        lines.push("Included files:".to_string());
        lines.extend(
            selection
                .included_files
                .iter()
                .map(|file| format!("- {file}")),
        );
    }
    if !selection.subassets.is_empty() {
        lines.push("Subassets:".to_string());
        lines.extend(selection.subassets.iter().map(|subasset| {
            format!(
                "- {} {} ({})",
                resource_kind_label(subasset.kind),
                subasset.locator,
                subasset.uuid
            )
        }));
    }
    lines.join("\n")
}

fn resource_kind_label(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Texture => "Texture",
        ResourceKind::Material => "Material",
        ResourceKind::Scene => "Scene",
        ResourceKind::Model => "Model",
        ResourceKind::Shader => "Shader",
        ResourceKind::Sound => "Sound",
        ResourceKind::Font => "Font",
        ResourceKind::PhysicsMaterial => "PhysicsMaterial",
        ResourceKind::NavMesh => "NavMesh",
        ResourceKind::NavigationSettings => "NavigationSettings",
        ResourceKind::Terrain => "Terrain",
        ResourceKind::TerrainLayerStack => "TerrainLayerStack",
        ResourceKind::TileSet => "TileSet",
        ResourceKind::TileMap => "TileMap",
        ResourceKind::Prefab => "Prefab",
        ResourceKind::AnimationSkeleton => "AnimationSkeleton",
        ResourceKind::AnimationClip => "AnimationClip",
        ResourceKind::AnimationSequence => "AnimationSequence",
        ResourceKind::AnimationGraph => "AnimationGraph",
        ResourceKind::AnimationStateMachine => "AnimationStateMachine",
        ResourceKind::UiLayout => "UiLayout",
        ResourceKind::UiWidget => "UiWidget",
        ResourceKind::UiStyle => "UiStyle",
        ResourceKind::Data => "Data",
        ResourceKind::MaterialGraph => "MaterialGraph",
    }
}
